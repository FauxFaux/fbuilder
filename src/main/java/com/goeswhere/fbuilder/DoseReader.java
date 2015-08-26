package com.goeswhere.fbuilder;

// ~/code/dose/deb-buildcheck.native --deb-native-arch=amd64 urika:3142_ftp.debian.org_debian_dists_sid_main_binary-amd64_Packages urika:3142_ftp.debian.org_debian_dists_sid_main_source_Sources -s -e > ~/xd

import com.google.common.base.Stopwatch;
import com.google.common.collect.Iterators;
import com.google.common.collect.PeekingIterator;
import org.yaml.snakeyaml.Yaml;
import org.yaml.snakeyaml.events.*;

import java.io.FileInputStream;
import java.io.InputStreamReader;
import java.util.Iterator;
import java.util.function.BiConsumer;

public class DoseReader {
    public static void main(String[] args) throws Exception {
        final Stopwatch timer = Stopwatch.createStarted();
        final PeekingIterator<Event> it = Iterators.peekingIterator(new Yaml().parse(new InputStreamReader(new FileInputStream(args[0]))).iterator());
        check(it.next() instanceof StreamStartEvent);
        check(it.next() instanceof DocumentStartEvent);

        readMap(it, (key, ev) -> {
            switch (key) {
                case "output-version":
                case "native-architecture":
                case "background-packages":
                case "foreground-packages":
                case "broken-packages":
//                    System.out.println(ev);
                    break;
                case "report":
                    check(ev instanceof SequenceStartEvent);
                    readSeq(it, () -> readMap(it, (skey, sev) -> {
                        switch (skey) {
                            case "package":
                            case "version":
                            case "architecture":
                            case "essential":
                            case "source":
                            case "status":
//                                    System.out.println(sev);
                                break;
                            case "installationset":
                                check(sev instanceof SequenceStartEvent);
                                readSeq(it, () -> readMap(it, (ikey, iev) -> {
//                                  System.out.println("ins: " + ikey + " -> " + iev);
                                }));
                                break;
                            default:
                                throw new IllegalStateException(skey);
                        }
                    }));
                    break;
                default:
                    throw new IllegalStateException(key);
            }
        });
        System.out.println(timer);
    }

    private static void readSeq(PeekingIterator<Event> it, Runnable elementHandler) {
        do {
            elementHandler.run();
        } while (it.peek() instanceof MappingStartEvent);
        check(it.next() instanceof SequenceEndEvent);
    }

    private static void readMap(Iterator<Event> it, BiConsumer<String, Event> callback) {
        check(it.next() instanceof MappingStartEvent);
        do {
            final Event next = it.next();
            if (next instanceof MappingEndEvent) {
                break;
            }
            callback.accept(((ScalarEvent) next).getValue(), it.next());
        } while (true);
    }

    private static void check(boolean b) {
        if (!b) {
            throw new IllegalStateException();
        }
    }
}

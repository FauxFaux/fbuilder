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
import java.util.function.Consumer;
import java.util.function.Function;

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
                    // ignored
                    break;
                case "report":
                    check(ev instanceof SequenceStartEvent);
                    readSeq(it, DoseReader::readSourcePackage, System.out::println);
                    break;
                default:
                    throw new IllegalStateException(key);
            }
        });
        System.out.println(timer);
    }

    private static SourcePackage readSourcePackage(PeekingIterator<Event> it) {
        final SourcePackage source = new SourcePackage();
        readMap(it, (key, ev) -> {
            switch (key) {
                case "package":
                    source.name = asString(ev);
                    break;
                case "version":
                    source.version = asString(ev);
                    break;
                case "architecture":
                case "essential":
                case "source":
                case "status":
                    // ignored
                    break;
                case "installationset":
                    check(ev instanceof SequenceStartEvent);
                    readSeq(it, DoseReader::readBinaryPackage, source.deps::add);
                    break;
                default:
                    throw new IllegalStateException(key);
            }
        });

        source.deps.remove(source.nameAndVersion());

        return source;
    }

    private static String asString(Event event) {
        check(event instanceof ScalarEvent);
        return ((ScalarEvent)event).getValue();
    }

    private static String readBinaryPackage(PeekingIterator<Event> it) {
        class NameVersion {
            String name;
            String version;

            @Override
            public String toString() {
                if (null == name || null == version) {
                    throw new IllegalStateException("name or version not set: " + name + ", " + version);
                }

                return name + "=" + version;
            }
        }

        final NameVersion pkg = new NameVersion();
        readMap(it, (key, ev) -> {
            switch (key) {
                case "package":
                    pkg.name = asString(ev);
                    break;
                case "version":
                    pkg.version = asString(ev);
                    break;
                case "architecture":
                case "essential":
                    // ignored
                    break;
                default:
                    throw new IllegalStateException(key);
            }
        });

        return pkg.name + "=" + pkg.version;
    }

    private static <T> void readSeq(
            PeekingIterator<Event> it,
            Function<PeekingIterator<Event>, ? extends T> elementHandler,
            Consumer<? super T> resultsInto) {
        do {
            resultsInto.accept(elementHandler.apply(it));
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
            callback.accept(asString(next), it.next());
        } while (true);
    }

    private static void check(boolean b) {
        if (!b) {
            throw new IllegalStateException();
        }
    }
}

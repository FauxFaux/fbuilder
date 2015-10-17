package com.goeswhere.fbuilder;

import com.google.common.io.Files;

import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.concurrent.*;

import static com.google.common.collect.Lists.newArrayList;

public class FBuilder {

    public static void main(String[] args) throws IOException, InterruptedException {
        if (args[0].startsWith("@")) {
            build(Files.readLines(new File(args[0].substring(1)), StandardCharsets.UTF_8));
        } else {
            build(Arrays.asList(args));
        }
    }

    private static void build(Iterable<String> in) throws IOException, InterruptedException {
        final List<String> args = newArrayList(in);
        Collections.sort(args);

        final int threads = Runtime.getRuntime().availableProcessors();

        final ExecutorService ex = Executors.newFixedThreadPool(threads);

        final String base = "base";
        new WithVm("base").createIfNotPresent();

        for (String pkg : args)
            ex.submit(() -> {
                final WithVm newVm = new WithVm("fbuild-" + pkg, TimeUnit.MINUTES.toMillis(30));

                final File rbuild = new File("wip-" + pkg + ".rbuild");
                try {
                    newVm.cloneFrom(base);
                    newVm.start();
                    newVm.inTee(rbuild, "apt-get", "-oAPT::Get::Only-Source=true", "source", pkg);
                    newVm.inTee(rbuild, "apt-get", "build-dep", "-y", "--force-yes", pkg);
                    newVm.inTee(rbuild, "ifdown", "eth0");
                    final boolean success = 0 == newVm.inTee(rbuild, "sh", "-c", "cd " + pkg + "-* && dpkg-buildpackage -us -uc");
                    newVm.stopNow();
                    if (success) {
                        rbuild.renameTo(new File("success-" + pkg + ".rbuild"));
                        newVm.destroy();
                        System.out.println("success: " + pkg);
                    } else {
                        rbuild.renameTo(new File("failure-" + pkg + ".rbuild"));
                        System.out.println("failure: " + pkg);
                    }
                } catch (Exception e) {
                    rbuild.renameTo(new File("error-" + pkg + ".rbuild"));
                    System.err.println("build error: " + pkg);
                    e.printStackTrace();
                    newVm.stopNow();
                }
                return null;
            });

        ex.shutdown();
        ex.awaitTermination(Long.MAX_VALUE, TimeUnit.MILLISECONDS);
    }
}

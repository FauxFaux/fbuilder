package com.goeswhere.fbuilder;

import java.io.*;
import java.util.concurrent.*;

public class FBuilder {

    public static void main(String[] args) throws IOException, InterruptedException {
        final int threads = Runtime.getRuntime().availableProcessors();

        final ExecutorService ex = Executors.newFixedThreadPool(threads);

        final String base = "base";
        new WithVm("base").createIfNotPresent();

        for (String pkg : args)
            ex.submit(() -> {
                final WithVm newVm = new WithVm("fbuild-" + pkg, TimeUnit.MINUTES.toMillis(10));

                final File rbuild = new File("wip-" + pkg + ".rbuild");
                try {
                    newVm.cloneFrom(base);
                    newVm.start();
                    newVm.inTee(rbuild, "apt-get", "-oAPT::Get::Only-Source=true", "source", pkg);
                    newVm.inTee(rbuild, "apt-get", "build-dep", "-y", pkg);
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

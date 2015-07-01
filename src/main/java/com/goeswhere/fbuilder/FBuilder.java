package com.goeswhere.fbuilder;

import com.google.common.base.Joiner;

import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.*;

public class FBuilder {
    public static void main(String[] args) throws IOException, InterruptedException {
        final int threads = Runtime.getRuntime().availableProcessors();

        final ExecutorService ex = new ThreadPoolExecutor(0, threads, 1, TimeUnit.MINUTES, new SynchronousQueue<>());
        final ScheduledExecutorService canceller = Executors.newScheduledThreadPool(1);

        final String base = "base";
        createIfNotPresent(base);

        for (String pkg : args) {
            final String newVm = "qbuild-" + pkg;

            final Future<Object> worker = ex.submit(() -> {
                try {
                    exec("lxc-clone", "-s", "-o", base, "-n", newVm);
                    start(newVm);
                    final File rbuild = new File("wip-" + pkg + ".rbuild");
                    inTee(newVm, rbuild, "apt-get", "build-dep", "-y", pkg);
                    inTee(newVm, rbuild, "apt-get", "source", pkg);
                    final boolean success = 0 == tee(rbuild, "lxc-attach", "-n", newVm, "--", "sh", "-c", "cd " + pkg + "-* && dpkg-buildpackage -us -uc");
                    stopNow(newVm);
                    if (success) {
                        rbuild.renameTo(new File("success-" + pkg + ".rbuild"));
                        destroy(newVm);
                        System.out.println("success: " + pkg);
                    } else {
                        rbuild.renameTo(new File("failure-" + pkg + ".rbuild"));
                        System.out.println("failure: " + pkg);
                    }
                } catch (Exception e) {
                    System.err.println("build failed: " + pkg);
                    e.printStackTrace();
                    stopNow(newVm);
                }
                return null;
            });

            canceller.schedule(() -> {
                if (!worker.isDone()) {
                    worker.cancel(true);
                }
            }, 10, TimeUnit.MINUTES);
        }

        ex.shutdown();
        ex.awaitTermination(Long.MAX_VALUE, TimeUnit.MILLISECONDS);
        canceller.shutdownNow();
    }

    private static void inTee(String vm, File rbuild, String... args) throws IOException, InterruptedException {
        tee(rbuild, l("lxc-attach", "-n", vm, "--").l(args).b());
    }

    private static int tee(File file, String... args) throws IOException, InterruptedException {
        final ProcessBuilder builder = setupExec(args);
        builder.redirectErrorStream(true);
        final Process proc = builder.start();
        proc.getOutputStream().close();
        final Thread copier = new Thread(() -> {

            try (final BufferedReader from = new BufferedReader(new InputStreamReader(proc.getInputStream(), StandardCharsets.UTF_8));
                 final PrintWriter to = new PrintWriter(new OutputStreamWriter(new FileOutputStream(file), StandardCharsets.UTF_8))) {
                String line;
                while (null != (line = from.readLine())) {
                    to.println(line);
                }
            } catch (IOException e) {
                e.printStackTrace();
            }
        });
        copier.setName("tee-" + file.getName());
        copier.start();
        copier.join();
        return proc.waitFor();
    }

    private static void createIfNotPresent(String base) throws IOException, InterruptedException {
        if (0 != execCode("lxc-info", "-n", base)) {
            exec("lxc-create", "-t", "download", "-B", "btrfs", "-n", base, "--", "-d", "debian", "-r", "sid", "-a", "amd64");
            start(base);
            shellIn(base, "printf " +
                    "'deb http://urika:9999/debian sid main contrib non-free\\n" +
                    "deb-src http://urika:9999/debian sid main contrib non-free'" +
                    " > /etc/apt/sources.list");
            in(base, "apt-get", "update");
            in(base, "apt-get", "dist-upgrade", "-y");
            in(base, "apt-get", "install", "-y", "build-essential");
            stopPolitely(base);
        }
    }

    private static void start(String vm) throws IOException, InterruptedException {
        exec("lxc-start", "-n", vm, "--logfile", "/tmp/a.log", "-l", "DEBUG");
        exec("lxc-wait", "-n", vm, "-s", "RUNNING");
        shellIn(vm, "while ! arp urika; do sleep 1; done");
    }

    private static void stopPolitely(String vm) throws IOException, InterruptedException {
        exec("lxc-stop", "-n", vm);
        exec("lxc-wait", "-n", vm, "-s", "STOPPED");
    }

    private static void stopNow(String newVm) throws IOException, InterruptedException {
        exec("lxc-stop", "-k", "-n", newVm);
    }

    private static void destroy(String newVm) throws IOException, InterruptedException {
        exec("lxc-destroy", "-n", newVm);
    }

    private static void shellIn(String vm, String command) throws IOException, InterruptedException {
        in(vm, "sh", "-c", command);
    }

    private static void in(String vm, String... args) throws IOException, InterruptedException {
        exec(l("lxc-attach", "-n", vm, "--").l(args).b());
    }

    private static void exec(String... cmd) throws IOException, InterruptedException {
        if (0 != execCode(cmd)) {
            throw new IllegalStateException("failed");
        }
    }

    private static ProcessBuilder setupExec(String... cmd) {
        System.out.println("$ " + Joiner.on(' ').join(cmd));
        final ProcessBuilder builder = new ProcessBuilder(cmd);
        builder.environment().put("LANG", "en_US.UTF-8");
        builder.environment().put("LANGUAGE", "en_US:en");
        builder.environment().put("TZ", "UTC");
        builder.environment().put("DEBIAN_FRONTEND", "noninteractive");
        return builder;
    }

    private static int execCode(String... cmd) throws IOException, InterruptedException {
        final ProcessBuilder builder = setupExec(cmd);
        final Process proc = builder
                .redirectOutput(ProcessBuilder.Redirect.INHERIT)
                .redirectError(ProcessBuilder.Redirect.INHERIT).start();
        proc.getOutputStream().close();
        return proc.waitFor();
    }

    private static ListBuilder l(String... args) {
        return new ListBuilder().l(args);
    }
}

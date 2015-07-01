package com.goeswhere.qfbuild;

import com.google.common.base.Joiner;

import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public class QfBuild {
    public static void main(String[] args) throws IOException, InterruptedException {
        final ExecutorService ex = Executors.newFixedThreadPool(Runtime.getRuntime().availableProcessors());

        final String base = "base";
        createIfNotPresent(base);


        in(base, l("apt-get", "build-dep", "-y").l(args).b());
        for (String pkg : args) {
            in(base, "apt-get", "source", pkg);
            tee(base, new File(pkg + ".rbuild"), "cd " + pkg + "-* && dpkg-buildpackage -us -uc");
        }
    }

    private static void tee(String base, File file, String shell) throws IOException, InterruptedException {
        final ProcessBuilder builder = setupExec("lxc-attach", "-n", base, "--", "sh", "-c", shell);
        builder.redirectErrorStream(true);
        final Process proc = builder.start();
        proc.getOutputStream().close();
        final Thread copier = new Thread(() -> {

            try (final BufferedReader from = new BufferedReader(new InputStreamReader(proc.getInputStream(), StandardCharsets.UTF_8));
                 final PrintWriter to = new PrintWriter(new OutputStreamWriter(new FileOutputStream(file), StandardCharsets.UTF_8))) {
                String line;
                while (null != (line = from.readLine())) {
                    System.out.println(line);
                    to.println(line);
                }
            } catch (IOException e) {
                e.printStackTrace();
            }
        });
        copier.setName("tee-"+ file.getName());
        copier.start();
        final int exitCode = proc.waitFor();
        copier.join();
        if (0 == exitCode) {
            System.out.println("build success so deleting rbuild");
            file.delete();
        } else {
            System.err.println("build failed");
        }
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
            stop(base);
        }
    }

    private static void snapshot(String base) throws IOException, InterruptedException {
        exec("lxc-snapshot", "-n", base);
    }

    private static void start(String base) throws IOException, InterruptedException {
        exec("lxc-start", "-n", base, "--logfile", "/tmp/a.log", "-l", "DEBUG");
        exec("lxc-wait", "-n", base, "-s", "RUNNING");
        shellIn(base, "while ! arp urika; do sleep 1; done");
    }

    private static void stop(String base) throws IOException, InterruptedException {
        exec("lxc-stop", "-n", base);
        exec("lxc-wait", "-n", base, "-s", "STOPPED");
    }

    private static void shellIn(String base, String command) throws IOException, InterruptedException {
        in(base, "sh", "-c", command);
    }

    private static void in(String base, String... args) throws IOException, InterruptedException {
        exec(l("lxc-attach", "-n", base, "--").l(args).b());
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

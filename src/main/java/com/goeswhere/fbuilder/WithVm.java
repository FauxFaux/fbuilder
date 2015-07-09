package com.goeswhere.fbuilder;

import com.google.common.base.Joiner;

import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.TimeUnit;

public class WithVm {
    private static final String HOSTNAME_TO_CHECK = "urika";

    private static final String MIRROR = "http://" + HOSTNAME_TO_CHECK + ":3142/ftp.debian.org/debian";

    final String vm;
    private final long mustBeDoneBy;

    public WithVm(String vm) {
        this(vm, TimeUnit.DAYS.toMillis(100));
    }

    public WithVm(String vm, long timeBudgetMillis) {
        this.vm = vm;
        mustBeDoneBy = System.currentTimeMillis() + timeBudgetMillis;
    }

    private static ListBuilder l(String... args) {
        return new ListBuilder().l(args);
    }

    int inTee(File rbuild, String... args) throws IOException, InterruptedException {
        return tee(rbuild, l("lxc-attach", "-n", vm, "--").l(args).b());
    }

    private int tee(File file, String... args) throws IOException, InterruptedException {
        final ProcessBuilder builder = setupExec(args);
        builder.redirectErrorStream(true);
        final Process proc = builder.start();
        proc.getOutputStream().close();
        final Thread copier = new Thread(() -> {

            try (final BufferedReader from = new BufferedReader(new InputStreamReader(proc.getInputStream(), StandardCharsets.UTF_8));
                 final PrintWriter to = new PrintWriter(new OutputStreamWriter(new FileOutputStream(file, true), StandardCharsets.UTF_8))) {
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
        copier.join(remainingTimeBudget());
        return waitFor(proc);
    }

    void createIfNotPresent() throws IOException, InterruptedException {
        if (0 != execCode("lxc-info", "-n", vm)) {
            exec("lxc-create", "-t", "download", "-B", "btrfs", "-n", vm, "--", "-d", "debian", "-r", "sid", "-a", "amd64");
            start();
            shellIn("printf " +
                    "'deb " + MIRROR + " sid main contrib non-free\\n" +
                    "deb-src " + MIRROR + " sid main contrib non-free'" +
                    " > /etc/apt/sources.list");
            in("apt-get", "update");
            in("apt-get", "dist-upgrade", "-y");
            in("apt-get", "install", "-y", "build-essential");
            stopPolitely();
        }
    }

    void start() throws IOException, InterruptedException {
        exec("lxc-start", "-n", vm, "--logfile", "/tmp/a.log", "-l", "DEBUG");
        exec("lxc-wait", "-n", vm, "-s", "RUNNING");
        shellIn("while ! arp " + HOSTNAME_TO_CHECK + "; do sleep 1; done");
    }

    private void stopPolitely() throws IOException, InterruptedException {
        exec("lxc-stop", "-n", vm);
        exec("lxc-wait", "-n", vm, "-s", "STOPPED");
    }

    void stopNow() throws IOException, InterruptedException {
        exec("lxc-stop", "-k", "-n", vm);
    }

    void destroy() throws IOException, InterruptedException {
        exec("lxc-destroy", "-n", vm);
    }

    private void shellIn(String command) throws IOException, InterruptedException {
        in("sh", "-c", command);
    }

    private void in(String... args) throws IOException, InterruptedException {
        exec(l("lxc-attach", "-n", vm, "--").l(args).b());
    }

    private void exec(String... cmd) throws IOException, InterruptedException {
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

    private int execCode(String... cmd) throws IOException, InterruptedException {
        final ProcessBuilder builder = setupExec(cmd);
        final Process proc = builder
                .redirectOutput(ProcessBuilder.Redirect.INHERIT)
                .redirectError(ProcessBuilder.Redirect.INHERIT).start();
        proc.getOutputStream().close();
        return waitFor(proc);
    }

    private int waitFor(Process proc) throws InterruptedException {
        if (!proc.waitFor(remainingTimeBudget(), TimeUnit.MILLISECONDS)) {
            throw new IllegalStateException("timeout");
        }
        return proc.exitValue();
    }

    private long remainingTimeBudget() {
        return mustBeDoneBy - System.currentTimeMillis();
    }

    public void cloneFrom(String base) throws IOException, InterruptedException {
        exec("lxc-clone", "-s", "-o", base, "-n", vm);
    }
}

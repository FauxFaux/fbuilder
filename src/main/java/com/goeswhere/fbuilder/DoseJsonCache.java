package com.goeswhere.fbuilder;

import com.fasterxml.jackson.core.JsonGenerationException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.google.common.base.Stopwatch;

import java.io.*;
import java.util.HashMap;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.TimeUnit;

import static com.goeswhere.fbuilder.DoseReader.readDebBuildCheckSourcePackages;

public class DoseJsonCache {

    private static final File CACHE_FILE;

    static {
        final File systemCacheDir;
        final String envCache = System.getenv("XDG_CACHE_HOME");
        if (null != envCache) {
            systemCacheDir = new File(envCache);
        } else {
            systemCacheDir = new File(System.getProperty("user.home"), ".cache");
        }

        final File ourCacheDir = new File(systemCacheDir, "fbuilder");
        if (!ourCacheDir.exists() && !ourCacheDir.mkdirs()) {
            throw new IllegalStateException("couldn't create cache directory: " + ourCacheDir);
        }

        CACHE_FILE = new File(ourCacheDir, "deps.json");
    }

    public static Map<String, Set<String>> loadCache() throws IOException {
        return new ObjectMapper().readValue(CACHE_FILE, new TypeReference<Map<String, Set<String>>>() {
        });
    }

    public static void main(String[] args) throws IOException {

        if (CACHE_FILE.lastModified() > System.currentTimeMillis() - TimeUnit.HOURS.toMillis(12)) {
            final Stopwatch stopwatch = Stopwatch.createStarted();
            final Map<String, Set<String>> map = loadCache();
            System.out.println(map.size());
            System.out.println(stopwatch.toString());
            return;
        }

        final InputStreamReader reader = new InputStreamReader(new FileInputStream(args[0]));

        final Map<String, Set<String>> deps = new HashMap<>();
        readDebBuildCheckSourcePackages(reader, sourcePackage -> {
            deps.put(sourcePackage.nameAndVersion(), sourcePackage.deps);
        });

        new ObjectMapper().writeValue(CACHE_FILE, deps);
    }
}

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

public class DoseToJson {

    private static final File CACHE_FILE = new File("a.json");

    public static void main(String[] args) throws IOException {

        if (CACHE_FILE.lastModified() > System.currentTimeMillis() - TimeUnit.HOURS.toMillis(12)) {
            final Stopwatch stopwatch = Stopwatch.createStarted();
            final Map<String, Set<String>> map = new ObjectMapper().readValue(CACHE_FILE, new TypeReference<Map<String, Set<String>>>() {
            });
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

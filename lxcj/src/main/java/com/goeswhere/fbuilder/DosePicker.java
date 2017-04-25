package com.goeswhere.fbuilder;

import java.io.IOException;
import java.util.HashSet;
import java.util.Iterator;
import java.util.Map;
import java.util.Set;

public class DosePicker {
    public static void main(String[] args) throws IOException {
        final Map<String, Set<String>> packages = DoseJsonCache.loadCache();
        System.out.println("Starting with: " + totalDeps(packages));
        Set<String> corePackages = findCorePackages(packages);
        for (Set<String> thisPackage : packages.values()) {
            thisPackage.removeAll(corePackages);
        }
        System.out.println("Eliminating build essential gives us: " + totalDeps(packages));
    }

    private static long totalDeps(Map<String, Set<String>> packages) {
        return packages.values().stream().mapToLong(Set::size).sum();
    }

    private static Set<String> findCorePackages(Map<String, Set<String>> packages) {
        final Iterator<Set<String>> it = packages.values().iterator();
        Set<String> initial = new HashSet<>(it.next());
        while (it.hasNext()) {
            initial.retainAll(it.next());
        }
        return initial;
    }

}

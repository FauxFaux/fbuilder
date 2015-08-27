package com.goeswhere.fbuilder;

import java.util.HashSet;
import java.util.Set;

public class SourcePackage {
    public String name;
    public String version;
    public Set<String> deps = new HashSet<>(100);

    public String nameAndVersion() {
        return name + "=" + version;
    }

    @Override
    public String toString() {
        return "SourcePackage{" +
                name + "=" + version +
                ", deps=" + deps +
                '}';
    }
}

package com.goeswhere.fbuilder;

import com.google.common.collect.ImmutableSet;

import java.util.HashSet;
import java.util.Objects;
import java.util.Set;

public class SourcePackage {
    public String name;
    public String version;
    public Set<String> deps = new HashSet<>(100);

    SourcePackage() {
    }

    public SourcePackage(String name, String version, Set<String> deps) {
        this.name = name;
        this.version = version;
        this.deps = ImmutableSet.copyOf(deps);
    }

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

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        SourcePackage that = (SourcePackage) o;
        return Objects.equals(name, that.name) &&
                Objects.equals(version, that.version) &&
                Objects.equals(deps, that.deps);
    }

    @Override
    public int hashCode() {
        return Objects.hash(name, version, deps);
    }
}

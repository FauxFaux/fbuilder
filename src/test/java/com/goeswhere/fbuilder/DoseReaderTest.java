package com.goeswhere.fbuilder;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import org.junit.Test;

import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.List;

import static org.junit.Assert.assertEquals;

public class DoseReaderTest {

    @Test
    public void testReadDebBuildCheckSourcePackages() throws Exception {
        List<SourcePackage> packages = new ArrayList<>();
        try (final InputStreamReader reader = new InputStreamReader(DoseReaderTest.class.getResourceAsStream("/deb-buildcheck-sample-tiny.yml"))) {
            DoseReader.readDebBuildCheckSourcePackages(reader, packages::add);
        }

        assertEquals(packages, ImmutableList.of(
                new SourcePackage("src:0ad", "0.0.18-1", ImmutableSet.of(
                        "base-files:amd64=9.4",
                        "adduser:amd64=3.113+nmu3",
                        "autoconf:amd64=2.69-9")),
                new SourcePackage("src:zzzeeksphinx", "1.0.17-1", ImmutableSet.of(
                        "zlib1g:amd64=1:1.2.8.dfsg-2+b1"))
        ));
    }
}
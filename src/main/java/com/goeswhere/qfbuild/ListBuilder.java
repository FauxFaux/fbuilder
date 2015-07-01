package com.goeswhere.qfbuild;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

class ListBuilder {

    List<String> things = new ArrayList<String>();
    public ListBuilder l(String... args) {
        Collections.addAll(things, args);
        return this;
    }

    public String[] b() {
        return things.toArray(new String[things.size()]);
    }
}

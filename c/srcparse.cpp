#include <apt-pkg/cachefile.h>
#include <apt-pkg/pkgcache.h>
#include <apt-pkg/srcrecords.h>

typedef pkgSrcRecords::Parser SrcPackage;
typedef std::vector<SrcPackage::BuildDepRec> BuildDepList;

int main() {
    // _config and _system are defined in the libapt header files
    pkgInitConfig(*_config);
    pkgInitSystem(*_config, _system);

    pkgCacheFile cache_file;
//    pkgCache* cache = cache_file.GetPkgCache();
    pkgSourceList *sources_list = cache_file.GetSourceList();
    pkgSrcRecords source_packages(*sources_list);

    source_packages.Restart();
    const SrcPackage *source_package_const;
    BuildDepList build_deps;
    while (NULL != (source_package_const = source_packages.Step())) {
        SrcPackage *source_package = const_cast<SrcPackage*>(source_package_const);
        build_deps.clear();
        source_package->BuildDepends(build_deps, true);
        std::cout << 
            source_package_const->Package() << "\t" <<
            source_package_const->Version() << "\t";
        for (BuildDepList::iterator it = build_deps.begin(); it != build_deps.end(); ++it) {
            std::cout << 
                it->Package << " " <<
                it->Version << " " <<
                it->Op << " " <<
                static_cast<int>(it->Type) << "\t";
        }

        std::cout << std::endl;
    }

    return 0;
}

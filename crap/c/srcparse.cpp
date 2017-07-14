#include <apt-pkg/cachefile.h>
#include <apt-pkg/cacheiterators.h>
#include <apt-pkg/pkgcache.h>
#include <apt-pkg/srcrecords.h>

typedef pkgSrcRecords::Parser SrcPackage;
typedef std::vector<SrcPackage::BuildDepRec> BuildDepList;

void src(pkgSourceList *sources_list);
void bin(pkgCache *binList);

void usage() {
    std::cout << "usage: -s|-b" << std::endl;
}

int main(int argc, char *argv[]) {

    if (2 != argc || '-' != argv[1][0] || strlen(argv[1]) != 2) {
        usage();
        return 1;
    }

    const char mode = argv[1][1];

    // _config and _system are defined in the libapt header files
    pkgInitConfig(*_config);
    pkgInitSystem(*_config, _system);

    pkgCacheFile cache_file;
//    pkgCache* cache = cache_file.GetPkgCache();

    if ('s' == mode) {
        src(cache_file.GetSourceList());
    }

    if ('b' == mode) {
        bin(cache_file.GetPkgCache());
    }
    return 0;
}

void src(pkgSourceList *sources_list) {
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
}

void bin(pkgCache *bin_list) {
    for (pkgCache::PkgIterator p = bin_list->PkgBegin(); p != bin_list->PkgEnd(); ++p) {
        std::cout << p.Name() << "\t";
        for (pkgCache::VerIterator v = p.VersionList(); !v.end(); ++v) {
            std::cout << v.Arch() << ":" << v.VerStr() << "\t";
            for (pkgCache::DepIterator d = v.DependsList(); !d.end(); ++d) {
                std::cout << d.DepType() << "\t";
                const char *cmp_type = d.CompType();
                if (cmp_type)
                    std::cout << "\t" << d.CompType() << "\t" << d.TargetVer() << "\t";
//                for (pkgCache::PkgIterator dp = d.SmartTargetPkg(); !dp.end(); ++dp) {
//                    std::cout << dp.Name() << "\t";
//                }
            }
        }
        std::cout << std::endl;
    }
}


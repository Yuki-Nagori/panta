// Home 页签的工程工具定义；Import 发语义信号，业务操作由 App 注入的 ViewModel 承接。
import QtQuick

RibbonContent {
    id: homeRibbon

    signal importRequested
    signal resultsRequested
    signal logsRequested

    groups: [
        {
            title: qsTranslate("UiCommon", "Import"),
            caret: false,
            tools: [
                {
                    key: "import",
                    label: qsTranslate("UiCommon", "Import"),
                    icon: "ribbon-import"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupCreate", "Create"),
            caret: false,
            tools: [
                {
                    key: "add",
                    label: qsTranslate("RibbonActionAdd", "Add"),
                    icon: "ribbon-add"
                },
                {
                    key: "dual-domain",
                    label: qsTranslate("RibbonActionDualDomain", "Dual\nDomain"),
                    icon: "ribbon-dual-domain",
                    caret: true
                },
                {
                    key: "geometry",
                    label: qsTranslate("UiCommonModeling", "Geometry"),
                    icon: "ribbon-geometry"
                },
                {
                    key: "mesh",
                    label: qsTranslate("UiCommonModeling", "Mesh"),
                    icon: "ribbon-mesh"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupSetup", "Molding Process Setup"),
            caret: false,
            tools: [
                {
                    key: "molding",
                    label: qsTranslate("RibbonActionMolding", "Thermoplastics\nInjection Molding"),
                    icon: "ribbon-thermoplastics-injection-molding"
                },
                {
                    key: "sequence",
                    label: qsTranslate("RibbonActionSequence", "Analysis\nSequence"),
                    icon: "ribbon-analysis-sequence"
                },
                {
                    key: "material",
                    label: qsTranslate("RibbonActionMaterial", "Select\nMaterial"),
                    icon: "ribbon-select-material"
                },
                {
                    key: "injection",
                    label: qsTranslate("RibbonActionInjection", "Injection\nLocations"),
                    icon: "ribbon-injection-locations"
                },
                {
                    key: "settings",
                    label: qsTranslate("RibbonActionSettings", "Process\nSettings"),
                    icon: "ribbon-process-settings"
                }
            ]
        },
        {
            title: qsTranslate("UiCommonResults", "Results"),
            caret: false,
            tools: [
                {
                    key: "optimization",
                    label: qsTranslate("UiCommonModeling", "Optimization"),
                    icon: "ribbon-optimization"
                },
                {
                    key: "boundary",
                    label: qsTranslate("RibbonActionBoundary", "Boundary\nConditions"),
                    icon: "ribbon-boundary-conditions"
                },
                {
                    key: "analyze",
                    label: qsTranslate("UiCommonAnalysis", "Analyze"),
                    icon: "ribbon-analyze",
                    enabled: false
                },
                {
                    key: "logs",
                    label: qsTranslate("RibbonActionLogs", "Logs"),
                    icon: "ribbon-logs"
                },
                {
                    key: "jobs",
                    label: qsTranslate("RibbonActionJobs", "Job\nManager"),
                    icon: "ribbon-job-manager"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupReporting", "Reporting"),
            caret: false,
            tools: [
                {
                    key: "results",
                    label: qsTranslate("UiCommonResults", "Results"),
                    icon: "ribbon-results"
                },
                {
                    key: "reports",
                    label: qsTranslate("UiCommonReports", "Reports"),
                    icon: "ribbon-reports"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupShare", "Share"),
            caret: false,
            tools: [
                {
                    key: "shared-views",
                    label: qsTranslate("RibbonActionSharedViews", "Shared\nViews"),
                    icon: "ribbon-shared-views"
                }
            ]
        }
    ]

    onActionRequested: key => {
        if (key === "import")
            homeRibbon.importRequested();
        else if (key === "results")
            homeRibbon.resultsRequested();
        else if (key === "logs")
            homeRibbon.logsRequested();
    }
}

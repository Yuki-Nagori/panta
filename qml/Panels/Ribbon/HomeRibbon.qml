// Home 页签的工程工具定义；Import 发语义信号，业务操作由 App 注入的 ViewModel 承接。
import QtQuick

RibbonContent {
    id: homeRibbon

    signal importRequested

    groups: [
        {
            title: qsTranslate("UiCommon", "Import"),
            caret: false,
            tools: [
                {
                    key: "import",
                    label: qsTranslate("UiCommon", "Import"),
                    icon: "project-import"
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
                    icon: "document-new"
                },
                {
                    key: "dual-domain",
                    label: qsTranslate("RibbonActionDualDomain", "Dual\nDomain"),
                    icon: "domain-dual",
                    caret: true
                },
                {
                    key: "geometry",
                    label: qsTranslate("UiCommonModeling", "Geometry"),
                    icon: "geometry"
                },
                {
                    key: "mesh",
                    label: qsTranslate("UiCommonModeling", "Mesh"),
                    icon: "mesh"
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
                    icon: "molding"
                },
                {
                    key: "sequence",
                    label: qsTranslate("RibbonActionSequence", "Analysis\nSequence"),
                    icon: "analysis-sequence"
                },
                {
                    key: "material",
                    label: qsTranslate("RibbonActionMaterial", "Select\nMaterial"),
                    icon: "material"
                },
                {
                    key: "injection",
                    label: qsTranslate("RibbonActionInjection", "Injection\nLocations"),
                    icon: "injection-location"
                },
                {
                    key: "settings",
                    label: qsTranslate("RibbonActionSettings", "Process\nSettings"),
                    icon: "process-settings"
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
                    icon: "optimization"
                },
                {
                    key: "boundary",
                    label: qsTranslate("RibbonActionBoundary", "Boundary\nConditions"),
                    icon: "boundary-conditions"
                },
                {
                    key: "analyze",
                    label: qsTranslate("UiCommonAnalysis", "Analyze"),
                    icon: "analysis-run",
                    enabled: false
                },
                {
                    key: "logs",
                    label: qsTranslate("RibbonActionLogs", "Logs"),
                    icon: "document-report"
                },
                {
                    key: "jobs",
                    label: qsTranslate("RibbonActionJobs", "Job\nManager"),
                    icon: "job-manager"
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
                    icon: "analysis-results"
                },
                {
                    key: "reports",
                    label: qsTranslate("UiCommonReports", "Reports"),
                    icon: "document-report"
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
                    icon: "shared-views"
                }
            ]
        }
    ]

    onActionRequested: key => {
        if (key === "import")
            homeRibbon.importRequested();
    }
}

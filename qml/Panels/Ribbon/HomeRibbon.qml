// Home 页签的工程工具定义；当前入口仍为视觉参考，尚未接入业务命令。
import QtQuick

RibbonContent {
    groups: [
        {
            title: qsTranslate("RibbonGroupImport", "Import"),
            caret: false,
            tools: [
                {
                    key: "import",
                    label: qsTranslate("RibbonActionImport", "Import"),
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
                    label: qsTranslate("RibbonActionGeometry", "Geometry"),
                    icon: "geometry"
                },
                {
                    key: "mesh",
                    label: qsTranslate("RibbonActionMesh", "Mesh"),
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
            title: qsTranslate("RibbonGroupAnalysis", "Results"),
            caret: false,
            tools: [
                {
                    key: "optimization",
                    label: qsTranslate("RibbonActionOptimization", "Optimization"),
                    icon: "optimization"
                },
                {
                    key: "boundary",
                    label: qsTranslate("RibbonActionBoundary", "Boundary\nConditions"),
                    icon: "boundary-conditions"
                },
                {
                    key: "analyze",
                    label: qsTranslate("RibbonActionAnalyze", "Analyze"),
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
                    label: qsTranslate("RibbonActionResults", "Results"),
                    icon: "analysis-results"
                },
                {
                    key: "reports",
                    label: qsTranslate("RibbonActionReports", "Reports"),
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
}

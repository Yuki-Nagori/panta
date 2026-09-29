// 开始页管理学习工具及已有工程命令；宿主负责对话框和服务调用。
import QtQuick

RibbonContent {
    id: startLearn

    signal newProjectRequested
    signal openProjectRequested

    groups: [
        {
            title: qsTranslate("RibbonGroupStart", "Start"),
            caret: true,
            tools: [
                {
                    key: "new-project",
                    label: qsTranslate("RibbonActionNewProject", "New\nProject"),
                    icon: "ribbon-project"
                },
                {
                    key: "open-project",
                    label: qsTranslate("RibbonActionOpenProject", "Open\nProject"),
                    icon: "ribbon-open-project"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupFeatures", "New Features"),
            caret: false,
            tools: [
                {
                    key: "new-features",
                    label: qsTranslate("RibbonActionNewFeatures", "New\nFeatures"),
                    icon: "ribbon-new-features"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupLearn", "Learn"),
            caret: false,
            tools: [
                {
                    key: "start-here",
                    label: qsTranslate("RibbonActionStartHere", "Start Here"),
                    icon: "ribbon-start-here"
                },
                {
                    key: "tutorials",
                    label: qsTranslate("RibbonActionTutorials", "Tutorials"),
                    icon: "ribbon-tutorials"
                },
                {
                    key: "videos",
                    label: qsTranslate("RibbonActionVideos", "Videos"),
                    icon: "ribbon-videos"
                },
                {
                    key: "help",
                    label: qsTranslate("UiCommonHelp", "Help"),
                    icon: "ribbon-help"
                }
            ]
        }
    ]

    onActionRequested: key => {
        if (key === "new-project")
            startLearn.newProjectRequested();
        else if (key === "open-project")
            startLearn.openProjectRequested();
    }
}

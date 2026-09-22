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
                    icon: "document-new"
                },
                {
                    key: "open-project",
                    label: qsTranslate("RibbonActionOpenProject", "Open\nProject"),
                    icon: "document-open"
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
                    icon: "ribbon-whatsnew"
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
                    icon: "ribbon-start"
                },
                {
                    key: "tutorials",
                    label: qsTranslate("RibbonActionTutorials", "Tutorials"),
                    icon: "ribbon-learn"
                },
                {
                    key: "videos",
                    label: qsTranslate("RibbonActionVideos", "Videos"),
                    icon: "media-video"
                },
                {
                    key: "help",
                    label: qsTranslate("UiCommonHelp", "Help"),
                    icon: "help-browser"
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

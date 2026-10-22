// 管理 Settings 页面路由，并打开 Dialogs 中的共享弹窗。
pragma ComponentBehavior: Bound

import QtQuick

Item {
    id: router

    objectName: "settingsPageRouter"
    width: 0
    height: 0

    property var registeredPages: [
        {
            routeId: "general",
            title: qsTranslate("SettingsDialog", "General"),
            iconName: "tools-options",
            preserveSourceColors: true,
            pageComponent: null
        },
        {
            routeId: "appearance",
            title: qsTranslate("SettingsDialog", "Appearance"),
            iconName: "task-settings",
            preserveSourceColors: true,
            pageComponent: null
        },
        {
            routeId: "language",
            title: qsTranslate("SettingsDialog", "Language"),
            iconName: "globe",
            preserveSourceColors: false,
            pageComponent: languagePageComponent
        }
    ]
    property string activeRouteId: "general"
    required property Window ownerWindow
    property var settingsService: null
    property string originalLanguage: ""
    property string candidateLanguage: ""
    property string languageError: ""

    signal accepted
    signal cancelled

    function registerPage(routeId, title, iconName, pageComponent, preserveSourceColors) {
        if (typeof routeId !== "string" || routeId.length === 0 || typeof title !== "string" || !pageComponent)
            return false;

        const pages = registeredPages.slice();
        const existingIndex = pages.findIndex(page => page.routeId === routeId);
        const existingPage = existingIndex < 0 ? null : pages[existingIndex];
        const page = {
            routeId: routeId,
            title: title,
            iconName: iconName || existingPage?.iconName || "history",
            preserveSourceColors: typeof preserveSourceColors === "boolean" ? preserveSourceColors : existingPage?.preserveSourceColors ?? false,
            pageComponent: pageComponent
        };
        if (existingIndex < 0)
            pages.push(page);
        else
            pages[existingIndex] = page;
        registeredPages = pages;
        return true;
    }

    function pageForRoute(routeId) {
        return registeredPages.find(page => page.routeId === routeId) || null;
    }

    function openPage(routeId) {
        const requestedPage = pageForRoute(routeId);
        const initialPage = requestedPage || pageForRoute("general") || registeredPages[0] || null;
        originalLanguage = settingsService ? settingsService.language : "";
        candidateLanguage = originalLanguage;
        languageError = "";
        activeRouteId = initialPage ? initialPage.routeId : "";
        settingsDialog.open();
        return Boolean(requestedPage);
    }

    function discardPendingLanguage() {
        candidateLanguage = originalLanguage;
        languageError = "";
    }

    function acceptPendingChanges() {
        if (settingsService && candidateLanguage.length > 0 && candidateLanguage !== originalLanguage) {
            if (!settingsService.setLanguage(candidateLanguage)) {
                languageError = qsTranslate("SettingsLanguagePage", "The selected language could not be applied. Your current language is unchanged.");
                candidateLanguage = originalLanguage;
                activeRouteId = "language";
                return;
            }
            originalLanguage = candidateLanguage;
        }
        settingsDialog.completeAccept();
    }

    Component {
        id: languagePageComponent

        LanguageSettingsPage {
            availableLanguages: router.settingsService ? router.settingsService.availableLanguages : []
            selectedLanguage: router.candidateLanguage
            errorText: router.languageError
            onLanguageSelected: locale => {
                router.candidateLanguage = locale;
                router.languageError = "";
            }
        }
    }

    SettingsDialog {
        id: settingsDialog
        ownerWindow: router.ownerWindow
        pages: router.registeredPages
        activeRouteId: router.activeRouteId
        onRouteSelected: routeId => router.activeRouteId = routeId
        onAcceptRequested: router.acceptPendingChanges()
        onAccepted: router.accepted()
        onCancelled: {
            router.discardPendingLanguage();
            router.cancelled();
        }
    }
}

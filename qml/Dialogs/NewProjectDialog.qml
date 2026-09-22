// 独立的新建工程窗口；工程命令通过显式 ProjectViewModel 注入。
import QtQuick
import QtQuick.Dialogs
import QtQuick.Layouts
import Panta.Bridge

Window {
    id: dialog
    objectName: "newProjectDialog"

    required property ProjectViewModel projectModel
    property Window ownerWindow
    signal projectCreated(string path)

    width: 620
    height: 360
    minimumWidth: 520
    minimumHeight: 320
    color: "transparent"
    modality: Qt.ApplicationModal
    flags: Qt.Dialog | Qt.FramelessWindowHint
    title: qsTranslate("NewProjectDialog", "Create New Project")
    transientParent: ownerWindow

    function resetFields() {
        projectModel.clearError();
        projectNameField.text = "";
        locationField.text = projectModel.defaultLocation;
    }

    function open() {
        resetFields();
        if (ownerWindow) {
            x = ownerWindow.x + Math.round((ownerWindow.width - width) / 2);
            y = ownerWindow.y + Math.round((ownerWindow.height - height) / 2);
        }
        visible = true;
        requestActivate();
        projectNameField.forceActiveFocus();
    }

    function submit() {
        if (projectModel.createProject(projectNameField.text, locationField.text)) {
            const path = projectModel.lastCreatedPath;
            dialog.projectCreated(path);
            dialog.close();
        } else {
            projectNameField.forceActiveFocus();
        }
    }

    function errorMessage() {
        if (projectModel.errorCode === "project.empty_name" || projectModel.errorCode === "project.invalid_name") {
            return qsTranslate("NewProjectDialog", "Enter a valid project name.");
        }
        if (projectModel.errorCode === "project.location_empty" || projectModel.errorCode === "project.location_not_absolute") {
            return qsTranslate("NewProjectDialog", "Choose an absolute project location.");
        }
        if (projectModel.errorCode === "project.location_create_failed") {
            return qsTranslate("NewProjectDialog", "The project location could not be created.");
        }
        if (projectModel.errorCode === "project.already_exists") {
            return qsTranslate("NewProjectDialog", "A project with this name already exists.");
        }
        if (projectModel.errorCode === "project.manifest_invalid" || projectModel.errorCode === "project.unsupported_schema") {
            return qsTranslate("NewProjectDialog", "The selected folder is not a supported panta project.");
        }
        if (projectModel.errorCode === "project.no_project") {
            return qsTranslate("NewProjectDialog", "Open or create a project first.");
        }
        if (projectModel.errorCode === "project.command_invalid") {
            return qsTranslate("NewProjectDialog", "The project change is not valid.");
        }
        return qsTranslate("NewProjectDialog", "The project operation could not be completed.");
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.colorPanel

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            DialogTitleBar {
                Layout.fillWidth: true
                window: dialog
                caption: dialog.title
                onCloseRequested: dialog.close()
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.leftMargin: Theme.spacingLarge
                Layout.rightMargin: Theme.spacingLarge
                Layout.topMargin: Theme.spacingMedium
                Layout.bottomMargin: Theme.spacingMedium
                spacing: Theme.spacingMedium

                ThemedLabel {
                    Layout.fillWidth: true
                    text: qsTranslate("NewProjectDialog", "Create a project directory and start with an empty workspace.")
                    textSize: Theme.fontSmall
                    textColor: Theme.colorTextMuted
                    wrapMode: Text.Wrap
                }

                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: Theme.spacingMedium
                    rowSpacing: Theme.spacingMedium

                    ThemedLabel {
                        text: qsTranslate("NewProjectDialog", "Project name")
                        textSize: Theme.fontBody
                    }
                    ThemedTextField {
                        id: projectNameField
                        objectName: "projectNameField"
                        Layout.fillWidth: true
                        Layout.preferredHeight: Theme.controlHeight
                        placeholderText: qsTranslate("NewProjectDialog", "My Project")
                        Accessible.name: qsTranslate("NewProjectDialog", "Project name")
                        invalid: projectModel.errorCode === "project.empty_name" || projectModel.errorCode === "project.invalid_name"
                        onAccepted: dialog.submit()
                    }

                    ThemedLabel {
                        text: qsTranslate("NewProjectDialog", "Create in")
                        textSize: Theme.fontBody
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.spacingSmall

                        ThemedTextField {
                            id: locationField
                            objectName: "projectLocationField"
                            Layout.fillWidth: true
                            Layout.preferredHeight: Theme.controlHeight
                            placeholderText: qsTranslate("NewProjectDialog", "Choose a folder")
                            Accessible.name: qsTranslate("NewProjectDialog", "Project location")
                            selectByMouse: true
                            invalid: projectModel.errorCode === "project.location_empty" || projectModel.errorCode === "project.location_not_absolute"
                        }
                        ThemedToolButton {
                            Layout.preferredWidth: 92
                            Layout.preferredHeight: Theme.controlHeight
                            text: qsTranslate("NewProjectDialog", "Browse")
                            iconName: "document-open"
                            contentPadding: Theme.spacingSmall
                            hoverColor: Theme.colorHover
                            borderColor: Theme.colorPanelLine
                            onClicked: folderDialog.open()
                        }
                    }
                }

                ThemedLabel {
                    Layout.fillWidth: true
                    visible: projectModel.error.length > 0
                    text: dialog.errorMessage()
                    textSize: Theme.fontSmall
                    textColor: Theme.colorError
                    wrapMode: Text.Wrap
                }

                ThemedLabel {
                    Layout.fillWidth: true
                    text: qsTranslate("NewProjectDialog", "The project folder and .panta file will be created if they do not already exist.")
                    textSize: Theme.fontSmall
                    textColor: Theme.colorTextMuted
                    wrapMode: Text.Wrap
                }

                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.spacingSmall

                    Item {
                        Layout.fillWidth: true
                    }

                    ThemedToolButton {
                        Layout.preferredWidth: 92
                        Layout.preferredHeight: Theme.controlHeight
                        text: qsTr("OK")
                        highlighted: true
                        contentColor: Theme.colorText
                        hoverColor: Theme.colorHover
                        borderColor: Theme.colorPanelLine
                        onClicked: dialog.submit()
                    }
                    ThemedToolButton {
                        Layout.preferredWidth: 92
                        Layout.preferredHeight: Theme.controlHeight
                        text: qsTr("Cancel")
                        contentColor: Theme.colorText
                        hoverColor: Theme.colorHover
                        borderColor: Theme.colorPanelLine
                        onClicked: dialog.close()
                    }
                }
            }
        }
    }

    FolderDialog {
        id: folderDialog
        title: qsTranslate("NewProjectDialog", "Choose Project Location")
        currentFolder: projectModel.defaultLocationUrl
        onAccepted: {
            const selectedPath = projectModel.localPath(selectedFolder);
            if (selectedPath.length > 0) {
                locationField.text = selectedPath;
                projectModel.clearError();
            }
        }
    }

    Shortcut {
        sequence: "Esc"
        onActivated: dialog.close()
    }
}

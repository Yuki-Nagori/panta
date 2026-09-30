// 浇口定位工艺候选；Rust 后台确认成功后由宿主关闭。
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Window {
    id: dialog
    objectName: "gateLocationSettingsDialog"
    property Window ownerWindow
    property var planSettings: ({})
    property var openedSettings: ({})
    property bool saving: false
    property string errorText: ""
    signal settingsRequested(string projectPath, double revision, string importId, var settings)
    readonly property bool canConfirm: !saving && !!openedSettings.importId && moldTemperature.acceptableInput && meltTemperature.acceptableInput && numberOfGates.acceptableInput
    width: Theme.gateLocationSettingsDialogWidth
    height: Theme.gateLocationSettingsDialogHeight
    minimumWidth: Theme.gateLocationSettingsDialogMinimumWidth
    minimumHeight: Theme.gateLocationSettingsDialogMinimumHeight
    color: Theme.colorTransparent
    modality: Qt.ApplicationModal
    flags: Qt.Dialog | Qt.FramelessWindowHint
    title: qsTranslate("GateLocationSettings", "Process Settings · Gate Location")
    transientParent: ownerWindow
    onClosing: event => {
        if (saving)
            event.accepted = false;
    }

    function open() {
        if (saving || planSettings.sequenceId !== "gate-location" || !planSettings.importId)
            return;
        openedSettings = planSettings;
        moldTemperature.text = String(openedSettings.gateLocationSettings.moldTemperature);
        meltTemperature.text = String(openedSettings.gateLocationSettings.meltTemperature);
        numberOfGates.text = String(openedSettings.gateLocationSettings.numberOfGates);
        if (ownerWindow) {
            x = ownerWindow.x + Math.round((ownerWindow.width - width) / 2);
            y = ownerWindow.y + Math.round((ownerWindow.height - height) / 2);
        }
        show();
        requestActivate();
        moldTemperature.forceActiveFocus();
    }
    function acceptSettings() {
        if (!visible || !canConfirm)
            return;
        const settings = openedSettings.gateLocationSettings;
        settingsRequested(openedSettings.projectPath, openedSettings.revision, openedSettings.importId, {
            machineId: settings.machineId,
            algorithmId: settings.algorithmId,
            moldTemperature: Number(moldTemperature.text),
            meltTemperature: Number(meltTemperature.text),
            numberOfGates: Number(numberOfGates.text)
        });
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
            ThemedLabel {
                Layout.fillWidth: true
                Layout.margins: Theme.spacingLarge
                visible: dialog.saving || dialog.errorText.length > 0
                text: dialog.saving ? qsTranslate("FillSettings", "Saving process settings…") : dialog.errorText
                textColor: dialog.saving ? Theme.colorTextMuted : Theme.colorError
                wrapMode: Text.Wrap
            }
            ScrollView {
                id: gateSettingsScroll
                enabled: !dialog.saving
                Layout.fillWidth: true
                Layout.fillHeight: true
                contentWidth: availableWidth
                clip: true
                ColumnLayout {
                    width: gateSettingsScroll.availableWidth
                    spacing: Theme.spacingLarge
                    ProcessSettingsControls.Section {
                        Layout.margins: Theme.spacingLarge
                        title: qsTranslate("GateLocationSettings", "Injection molding machine")
                        RowLayout {
                            anchors.fill: parent
                            ProcessSettingsControls.Choice {
                                model: [qsTranslate("GateLocationSettings", dialog.openedSettings.gateLocationSettings?.machineSourceText ?? "")]
                                Accessible.name: qsTranslate("GateLocationSettings", "Injection molding machine")
                            }
                            ProcessSettingsControls.Action {
                                text: qsTranslate("GateLocationSettings", "Edit…")
                                enabled: false
                            }
                            ProcessSettingsControls.Action {
                                text: qsTranslate("GateLocationSettings", "Select…")
                                enabled: false
                            }
                        }
                    }
                    ProcessSettingsControls.Section {
                        Layout.leftMargin: Theme.spacingLarge
                        Layout.rightMargin: Theme.spacingLarge
                        title: qsTranslate("FillSettings", "Temperatures")
                        GridLayout {
                            anchors.fill: parent
                            columns: 2
                            uniformCellWidths: true
                            columnSpacing: Theme.spacingLarge
                            ThemedLabel {
                                text: qsTranslate("FillSettings", "Mold surface temperature")
                            }
                            ThemedLabel {
                                text: qsTranslate("FillSettings", "Melt temperature")
                            }
                            RowLayout {
                                Layout.fillWidth: true
                                Layout.preferredWidth: 0
                                ProcessSettingsControls.ValueField {
                                    id: moldTemperature
                                    objectName: "gateMoldTemperature"
                                    Accessible.name: qsTranslate("FillSettings", "Mold surface temperature")
                                    validator: DoubleValidator {
                                        bottom: -273.15
                                        locale: "C"
                                        notation: DoubleValidator.StandardNotation
                                    }
                                }
                                ThemedLabel {
                                    text: "°C"
                                    textColor: Theme.colorTextMuted
                                }
                            }
                            RowLayout {
                                Layout.fillWidth: true
                                Layout.preferredWidth: 0
                                ProcessSettingsControls.ValueField {
                                    id: meltTemperature
                                    objectName: "gateMeltTemperature"
                                    Accessible.name: qsTranslate("FillSettings", "Melt temperature")
                                    validator: DoubleValidator {
                                        bottom: -273.15
                                        locale: "C"
                                        notation: DoubleValidator.StandardNotation
                                    }
                                }
                                ThemedLabel {
                                    text: "°C"
                                    textColor: Theme.colorTextMuted
                                }
                            }
                        }
                    }
                    ProcessSettingsControls.Section {
                        Layout.leftMargin: Theme.spacingLarge
                        Layout.rightMargin: Theme.spacingLarge
                        Layout.bottomMargin: Theme.spacingLarge
                        title: qsTranslate("GateLocationSettings", "Gate locator")
                        ColumnLayout {
                            anchors.fill: parent
                            spacing: Theme.spacingLarge
                            GridLayout {
                                columns: 2
                                Layout.fillWidth: true
                                columnSpacing: Theme.spacingLarge
                                ThemedLabel {
                                    text: qsTranslate("GateLocationSettings", "Algorithm")
                                }
                                ThemedLabel {
                                    text: qsTranslate("GateLocationSettings", "Number of gates")
                                }
                                ProcessSettingsControls.Choice {
                                    model: [qsTranslate("GateLocationSettings", dialog.openedSettings.gateLocationSettings?.algorithmSourceText ?? "")]
                                    Accessible.name: qsTranslate("GateLocationSettings", "Algorithm")
                                }
                                RowLayout {
                                    Layout.preferredWidth: Theme.processValueColumnWidth
                                    ProcessSettingsControls.ValueField {
                                        id: numberOfGates
                                        objectName: "gateNumberOfGates"
                                        Accessible.name: qsTranslate("GateLocationSettings", "Number of gates")
                                        validator: IntValidator {
                                            bottom: 1
                                            top: 10
                                            locale: "C"
                                        }
                                    }
                                    ThemedLabel {
                                        text: "1–10"
                                        textColor: Theme.colorTextMuted
                                    }
                                }
                            }
                            ProcessSettingsControls.Action {
                                text: qsTranslate("FillSettings", "Advanced Options…")
                                enabled: false
                            }
                        }
                    }
                }
            }
            RowLayout {
                Layout.fillWidth: true
                Layout.margins: Theme.spacingLarge
                Item {
                    Layout.fillWidth: true
                }
                ProcessSettingsControls.Action {
                    objectName: "gateLocationSettingsAccept"
                    Layout.preferredWidth: Theme.tabSegmentWidth
                    controlHeight: Theme.controlHeight
                    text: qsTranslate("DialogAction", "OK")
                    enabled: dialog.canConfirm
                    primaryAction: true
                    borderColor: Theme.colorDialogPrimaryBorder
                    onClicked: dialog.acceptSettings()
                }
                ProcessSettingsControls.Action {
                    objectName: "gateLocationSettingsCancel"
                    Layout.preferredWidth: Theme.tabSegmentWidth
                    controlHeight: Theme.controlHeight
                    text: qsTranslate("DialogAction", "Cancel")
                    enabled: !dialog.saving
                    onClicked: dialog.close()
                }
                ProcessSettingsControls.Action {
                    Layout.preferredWidth: Theme.tabSegmentWidth
                    controlHeight: Theme.controlHeight
                    text: qsTranslate("UiCommonHelp", "Help")
                    enabled: false
                }
            }
        }
    }
    Shortcut {
        sequence: "Esc"
        enabled: dialog.visible && !dialog.saving
        onActivated: dialog.close()
    }
}

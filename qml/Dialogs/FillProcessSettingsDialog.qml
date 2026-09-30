// Fill 工艺设置候选；目标修订冻结，Rust 后台确认成功后由宿主关闭。
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Window {
    id: dialog
    objectName: "fillProcessSettingsDialog"
    property Window ownerWindow
    property var planSettings: ({})
    property var profile: []
    property var openedSettings: ({})
    property bool saving: false
    property string errorText: ""
    signal settingsRequested(string projectPath, double revision, string importId, var settings)
    readonly property bool canConfirm: !saving && !!openedSettings.importId && profile.length > 0 && moldTemperature.acceptableInput && meltTemperature.acceptableInput && flowRate.acceptableInput && Number(flowRate.text) > 0 && switchVolume.acceptableInput
    width: Theme.fillSettingsDialogWidth
    height: Theme.fillSettingsDialogHeight
    minimumWidth: Theme.fillSettingsDialogMinimumWidth
    minimumHeight: Theme.fillSettingsDialogMinimumHeight
    color: Theme.colorTransparent
    modality: Qt.ApplicationModal
    flags: Qt.Dialog | Qt.FramelessWindowHint
    title: qsTranslate("FillSettings", "Process Settings · Fill")
    transientParent: ownerWindow
    onClosing: event => {
        if (saving)
            event.accepted = false;
        else
            profileDialog.close();
    }

    function open() {
        if (saving || planSettings.sequenceId !== "fill" || !planSettings.importId)
            return;
        openedSettings = planSettings;
        const settings = openedSettings.fillSettings;
        moldTemperature.text = String(settings.moldTemperature);
        meltTemperature.text = String(settings.meltTemperature);
        flowRate.text = String(settings.flowRate);
        switchVolume.text = String(settings.switchVolume);
        fiberOrientation.checked = settings.fiberOrientation;
        crystallization.checked = settings.crystallization;
        profile = settings.holdingProfile.map(point => ({
                    duration: point.duration,
                    pressure: point.pressure
                }));
        if (ownerWindow) {
            x = ownerWindow.x + Math.round((ownerWindow.width - width) / 2);
            y = ownerWindow.y + Math.round((ownerWindow.height - height) / 2);
        }
        show();
        requestActivate();
        moldTemperature.forceActiveFocus();
    }

    function acceptSettings() {
        if (!visible || !canConfirm || profileDialog.visible)
            return;
        settingsRequested(openedSettings.projectPath, openedSettings.revision, openedSettings.importId, {
            moldTemperature: Number(moldTemperature.text),
            meltTemperature: Number(meltTemperature.text),
            flowRate: Number(flowRate.text),
            switchVolume: Number(switchVolume.text),
            fiberOrientation: fiberOrientation.checked,
            crystallization: crystallization.checked,
            holdingProfile: profile
        });
    }

    component Option: CheckBox {
        id: option
        padding: 0
        spacing: Theme.spacingSmall
        indicator: Rectangle {
            implicitWidth: Theme.iconSizeSmall
            implicitHeight: Theme.iconSizeSmall
            y: (option.height - height) / 2
            radius: Theme.radiusSmall
            color: option.checked ? Theme.colorFocus : Theme.colorPanel
            border.color: option.visualFocus ? Theme.colorFocus : Theme.colorPanelLine
            border.width: option.visualFocus ? Theme.focusBorderWidth : Theme.borderWidth
            ThemedLabel {
                anchors.centerIn: parent
                text: "✓"
                visible: option.checked
                textColor: Theme.colorPanel
                textSize: Theme.fontSmall
            }
        }
        contentItem: ThemedLabel {
            text: option.text
            leftPadding: option.indicator.width + option.spacing
            wrapMode: Text.Wrap
            verticalAlignment: Text.AlignVCenter
        }
    }

    HoldingProfileDialog {
        id: profileDialog
        ownerWindow: dialog
        onProfileAccepted: points => dialog.profile = points
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
                id: fillSettingsScroll
                enabled: !dialog.saving
                Layout.fillWidth: true
                Layout.fillHeight: true
                contentWidth: availableWidth
                clip: true
                ColumnLayout {
                    width: fillSettingsScroll.availableWidth
                    spacing: Theme.spacingLarge
                    ProcessSettingsControls.Section {
                        Layout.margins: Theme.spacingLarge
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
                                    objectName: "moldSurfaceTemperature"
                                    Accessible.name: qsTranslate("FillSettings", "Mold surface temperature")
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
                                    objectName: "meltTemperature"
                                    Accessible.name: qsTranslate("FillSettings", "Melt temperature")
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
                        title: qsTranslate("FillSettings", "Filling & switch-over")
                        GridLayout {
                            anchors.fill: parent
                            columns: 3
                            rowSpacing: Theme.spacingLarge
                            columnSpacing: Theme.spacingLarge
                            ThemedLabel {
                                text: qsTranslate("FillSettings", "Filling control")
                            }
                            ProcessSettingsControls.Choice {
                                model: [qsTranslate("FillSettings", "Flow rate")]
                                Accessible.name: qsTranslate("FillSettings", "Filling control")
                            }
                            RowLayout {
                                Layout.preferredWidth: Theme.processValueColumnWidth
                                ProcessSettingsControls.ValueField {
                                    id: flowRate
                                    objectName: "fillFlowRate"
                                    Accessible.name: qsTranslate("FillSettings", "Flow rate")
                                    validator: DoubleValidator {
                                        bottom: 0
                                        locale: "C"
                                        notation: DoubleValidator.StandardNotation
                                    }
                                }
                                ThemedLabel {
                                    text: "cm³/s"
                                    textColor: Theme.colorTextMuted
                                }
                            }
                            ThemedLabel {
                                text: qsTranslate("FillSettings", "Velocity/pressure switch-over")
                                wrapMode: Text.Wrap
                                Layout.maximumWidth: Theme.processValueColumnWidth + Theme.tabSegmentWidth
                            }
                            ProcessSettingsControls.Choice {
                                model: [qsTranslate("FillSettings", "By volume filled")]
                                Accessible.name: qsTranslate("FillSettings", "Velocity/pressure switch-over")
                            }
                            RowLayout {
                                ProcessSettingsControls.ValueField {
                                    id: switchVolume
                                    objectName: "switchOverVolume"
                                    Accessible.name: qsTranslate("FillSettings", "Volume filled")
                                    validator: DoubleValidator {
                                        bottom: 0
                                        top: 100
                                        locale: "C"
                                        notation: DoubleValidator.StandardNotation
                                    }
                                }
                                ThemedLabel {
                                    text: "%"
                                    textColor: Theme.colorTextMuted
                                }
                            }
                        }
                    }
                    ProcessSettingsControls.Section {
                        Layout.leftMargin: Theme.spacingLarge
                        Layout.rightMargin: Theme.spacingLarge
                        title: qsTranslate("FillSettings", "Pack/holding control")
                        ColumnLayout {
                            anchors.fill: parent
                            RowLayout {
                                ProcessSettingsControls.Choice {
                                    model: [qsTranslate("FillSettings", "Filling pressure vs time (%)")]
                                    Accessible.name: qsTranslate("FillSettings", "Pack/holding control")
                                }
                                ProcessSettingsControls.Action {
                                    objectName: "editHoldingProfile"
                                    text: qsTranslate("FillSettings", "Edit Profile…")
                                    onClicked: profileDialog.open(dialog.profile)
                                }
                            }
                            ThemedLabel {
                                text: qsTranslate("FillSettings", "%1 steps · %2 s").arg(dialog.profile.length).arg(dialog.profile.reduce((total, point) => total + point.duration, 0))
                                textColor: Theme.colorTextMuted
                            }
                        }
                    }
                    ProcessSettingsControls.Section {
                        Layout.leftMargin: Theme.spacingLarge
                        Layout.rightMargin: Theme.spacingLarge
                        Layout.bottomMargin: Theme.spacingLarge
                        title: qsTranslate("FillSettings", "Analysis options")
                        ColumnLayout {
                            anchors.fill: parent
                            spacing: Theme.spacingLarge
                            Option {
                                id: fiberOrientation
                                Layout.fillWidth: true
                                text: qsTranslate("FillSettings", "Fiber orientation analysis if fiber material")
                            }
                            Option {
                                id: crystallization
                                Layout.fillWidth: true
                                text: qsTranslate("FillSettings", "Crystallization analysis (requires material data)")
                            }
                            RowLayout {
                                ProcessSettingsControls.Action {
                                    text: qsTranslate("FillSettings", "Advanced Options…")
                                    enabled: false
                                }
                                ProcessSettingsControls.Action {
                                    text: qsTranslate("FillSettings", "Fiber Solver Parameters…")
                                    enabled: false
                                }
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
                    objectName: "fillSettingsAccept"
                    Layout.preferredWidth: Theme.tabSegmentWidth
                    controlHeight: Theme.controlHeight
                    text: qsTranslate("DialogAction", "OK")
                    enabled: dialog.canConfirm
                    primaryAction: true
                    borderColor: Theme.colorDialogPrimaryBorder
                    onClicked: dialog.acceptSettings()
                }
                ProcessSettingsControls.Action {
                    Layout.preferredWidth: Theme.tabSegmentWidth
                    controlHeight: Theme.controlHeight
                    objectName: "fillSettingsCancel"
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
        enabled: dialog.visible && !dialog.saving && !profileDialog.visible
        onActivated: dialog.close()
    }
}

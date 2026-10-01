// Fill 工艺设置候选；目标修订冻结，Rust 后台确认成功后由宿主关闭。
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

DialogWindow {
    id: dialog
    objectName: "fillProcessSettingsDialog"
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
    title: qsTranslate("FillSettings", "Process Settings · Fill")
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
        centerOnOwner();
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

    HoldingProfileDialog {
        id: profileDialog
        ownerWindow: dialog
        onProfileAccepted: points => dialog.profile = points
    }
    DialogFrame {
        window: dialog
        closeEnabled: true

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
                FormSection {
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
                            ThemedNumberField {
                                id: moldTemperature
                                Layout.fillWidth: true
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
                            ThemedNumberField {
                                id: meltTemperature
                                Layout.fillWidth: true
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
                FormSection {
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
                        ThemedComboBox {
                            Layout.fillWidth: true

                            font.pixelSize: Theme.fontBody
                            model: [qsTranslate("FillSettings", "Flow rate")]
                            Accessible.name: qsTranslate("FillSettings", "Filling control")
                        }
                        RowLayout {
                            Layout.preferredWidth: Theme.processValueColumnWidth
                            ThemedNumberField {
                                id: flowRate
                                Layout.fillWidth: true
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
                        ThemedComboBox {
                            Layout.fillWidth: true

                            font.pixelSize: Theme.fontBody
                            model: [qsTranslate("FillSettings", "By volume filled")]
                            Accessible.name: qsTranslate("FillSettings", "Velocity/pressure switch-over")
                        }
                        RowLayout {
                            ThemedNumberField {
                                id: switchVolume
                                Layout.fillWidth: true
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
                FormSection {
                    Layout.leftMargin: Theme.spacingLarge
                    Layout.rightMargin: Theme.spacingLarge
                    title: qsTranslate("FillSettings", "Pack/holding control")
                    ColumnLayout {
                        anchors.fill: parent
                        RowLayout {
                            ThemedComboBox {
                                Layout.fillWidth: true

                                font.pixelSize: Theme.fontBody
                                model: [qsTranslate("FillSettings", "Filling pressure vs time (%)")]
                                Accessible.name: qsTranslate("FillSettings", "Pack/holding control")
                            }
                            ThemedButton {
                                contentPadding: Theme.spacingLarge
                                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
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
                FormSection {
                    Layout.leftMargin: Theme.spacingLarge
                    Layout.rightMargin: Theme.spacingLarge
                    Layout.bottomMargin: Theme.spacingLarge
                    title: qsTranslate("FillSettings", "Analysis options")
                    ColumnLayout {
                        anchors.fill: parent
                        spacing: Theme.spacingLarge
                        ThemedCheckBox {
                            id: fiberOrientation
                            wrapText: true
                            Layout.fillWidth: true
                            text: qsTranslate("FillSettings", "Fiber orientation analysis if fiber material")
                        }
                        ThemedCheckBox {
                            id: crystallization
                            wrapText: true
                            Layout.fillWidth: true
                            text: qsTranslate("FillSettings", "Crystallization analysis (requires material data)")
                        }
                        RowLayout {
                            ThemedButton {
                                contentPadding: Theme.spacingLarge
                                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                                text: qsTranslate("FillSettings", "Advanced Options…")
                                enabled: false
                            }
                            ThemedButton {
                                contentPadding: Theme.spacingLarge
                                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                                text: qsTranslate("FillSettings", "Fiber Solver Parameters…")
                                enabled: false
                            }
                        }
                    }
                }
            }
        }
        DialogButtonRow {
            Layout.fillWidth: true
            Layout.margins: Theme.spacingLarge

            ThemedButton {
                contentPadding: Theme.spacingLarge
                objectName: "fillSettingsAccept"

                text: qsTranslate("DialogAction", "OK")
                enabled: dialog.canConfirm
                primaryAction: true
                onClicked: dialog.acceptSettings()
            }
            ThemedButton {
                contentPadding: Theme.spacingLarge

                objectName: "fillSettingsCancel"
                text: qsTranslate("DialogAction", "Cancel")
                enabled: !dialog.saving
                onClicked: dialog.close()
            }
            ThemedButton {
                contentPadding: Theme.spacingLarge
                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)

                text: qsTranslate("UiCommonHelp", "Help")
                enabled: false
            }
        }
    }
    Shortcut {
        sequence: "Esc"
        enabled: dialog.visible && !dialog.saving && !profileDialog.visible
        onActivated: dialog.close()
    }
}

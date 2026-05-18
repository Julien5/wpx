import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:wpx/src/widgets/small.dart';

enum SpeedMode { constant, acp }

// Pick speeds relevant to randonneuring.
/*
  https://www.randonneursmondiaux.org/files/LRM_Event_Regulations_2023.pdf
  https://rusa.org/pages/acp-brevet-control-times-calculator
  https://www.audax-club-parisien.com/organisation/brm-monde/#reglement-BRM
  https://www.audax-club-parisien.com/download/plages_horaires_brm_10_FR.xls
*/

SpeedMode parseSpeedMode(String speed) {
  if (speed.toUpperCase() == "ACP") {
    return SpeedMode.acp;
  }
  return SpeedMode.constant;
}

String speedModeToString(SpeedMode mode, String constantValue) {
  if (mode == SpeedMode.acp) {
    return "ACP";
  }
  return constantValue;
}

void openSpeedDialog({
  required BuildContext context,
  required String speed,
  required String? initialConstantSpeed,
  required Function(String) onSpeedChanged,
  required Function(String) onConfirm,
  required Function(String) onCancel,
}) {
  SpeedMode initialMode = parseSpeedMode(speed);
  SpeedMode currentMode = initialMode;
  String currentSpeed = speedModeToString(
    initialMode,
    initialConstantSpeed ?? "15.0",
  );
  TextEditingController textController = TextEditingController(
    text: initialConstantSpeed ?? "15.0",
  );

  void adjustSpeed(double delta) {
    if (currentMode != SpeedMode.constant) return;

    double currentValue = double.tryParse(textController.text) ?? 15.0;
    double newValue = (currentValue + delta).clamp(1.0, 100.0);
    // Round to 1 decimal place
    newValue = (newValue * 10).round() / 10;

    textController.text = newValue.toString();
    currentSpeed = newValue.toString();
  }

  final FocusNode textFieldFocusNode = FocusNode(
    onKeyEvent: (node, event) {
      if (event is KeyDownEvent) {
        if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
          adjustSpeed(0.1);
          return KeyEventResult.handled;
        } else if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
          adjustSpeed(-0.1);
          return KeyEventResult.handled;
        }
      }

      if (event.logicalKey == LogicalKeyboardKey.arrowLeft ||
          event.logicalKey == LogicalKeyboardKey.arrowRight) {
        // Let the TextField handle cursor movement; stop propagation to RadioGroup.
        return KeyEventResult.skipRemainingHandlers;
      }
      return KeyEventResult.ignored;
    },
  );
  showDialog(
    context: context,
    builder: (BuildContext context) {
      return StatefulBuilder(
        builder: (context, setDialogState) {
          return StandardDialog(
            sections: [
              // Header
              DialogHeader(label: 'SELECT SPEED', title: null),
              // Mode section
              Padding(
                padding: DialogStyles.contentPadding,
                child: RadioGroup<SpeedMode>(
                  groupValue: currentMode,
                  onChanged: (SpeedMode? value) {
                    if (value != null) {
                      setDialogState(() {
                        currentMode = value;
                      });
                      // When switching to constant mode, use the text field value
                      // When switching to ACP, use "ACP"
                      if (value == SpeedMode.constant) {
                        currentSpeed =
                            textController.text.isNotEmpty
                                ? textController.text
                                : "15.0";
                      } else {
                        currentSpeed = "ACP";
                      }
                    }
                  },
                  child: Column(
                    children: [
                      RadioListTile<SpeedMode>(
                        title: const Text("Average speed"),
                        value: SpeedMode.constant,
                        controlAffinity: ListTileControlAffinity.leading,
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 16.0,
                          vertical: 8.0,
                        ),
                        child: Row(
                          children: [
                            Expanded(
                              child: Listener(
                                onPointerSignal: (event) {
                                  if (event is PointerScrollEvent &&
                                      currentMode == SpeedMode.constant) {
                                    // Scroll up (negative delta) increases speed
                                    // Scroll down (positive delta) decreases speed
                                    double delta =
                                        event.scrollDelta.dy > 0 ? -1 : 1;
                                    adjustSpeed(delta);
                                  }
                                },
                                child: TextField(
                                  focusNode: textFieldFocusNode,
                                  controller: textController,
                                  enabled: currentMode == SpeedMode.constant,
                                  keyboardType:
                                      const TextInputType.numberWithOptions(
                                        decimal: true,
                                      ),
                                  inputFormatters: [
                                    FilteringTextInputFormatter.allow(
                                      RegExp(r'^\d*\.?\d{0,3}'),
                                    ),
                                  ],
                                  decoration: const InputDecoration(
                                    labelText: 'Speed (km/h)',
                                    border: OutlineInputBorder(),
                                    suffixText: 'km/h',
                                  ),
                                  onChanged: (value) {
                                    if (value.isNotEmpty &&
                                        double.tryParse(value) != null) {
                                      double parsedValue = double.parse(value);
                                      double clampedValue = parsedValue.clamp(
                                        1.0,
                                        100.0,
                                      );
                                      currentSpeed = clampedValue.toString();
                                    }
                                  },
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                      const SizedBox(height: 8),
                      RadioListTile<SpeedMode>(
                        title: const Text("ACP"),
                        value: SpeedMode.acp,
                        controlAffinity: ListTileControlAffinity.leading,
                      ),
                    ],
                  ),
                ),
              ),
              // Footer
              DialogFooter(
                onCancel: () {
                  onCancel(currentSpeed);
                  Navigator.of(context).pop();
                },
                onConfirm: () {
                  onConfirm(currentSpeed);
                  Navigator.of(context).pop();
                },
              ),
            ],
          );
        },
      );
    },
  );
}

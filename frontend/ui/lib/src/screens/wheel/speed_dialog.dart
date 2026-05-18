import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

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
  String currentSpeed = initialConstantSpeed ?? "15";
  TextEditingController textController = TextEditingController(
    text: currentSpeed,
  );
  Timer? debounceTimer;
  final FocusNode textFieldFocusNode = FocusNode(
    onKeyEvent: (node, event) {
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
          return SimpleDialog(
            title: const Text('Speed', textAlign: TextAlign.center),
            contentPadding: const EdgeInsets.all(16.0),
            children: [
              RadioGroup<SpeedMode>(
                groupValue: currentMode,
                onChanged: (SpeedMode? value) {
                  if (value != null) {
                    setDialogState(() {
                      currentMode = value;
                    });
                    currentSpeed = speedModeToString(value, currentSpeed);
                  }
                },
                child: Column(
                  children: [
                    RadioListTile<SpeedMode>(
                      title: const Text("Constant speed"),
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
                                  debounceTimer?.cancel();
                                  debounceTimer = Timer(
                                    const Duration(milliseconds: 250),
                                    () {
                                      String newSpeed = speedModeToString(
                                        currentMode,
                                        currentSpeed,
                                      );
                                      onSpeedChanged(newSpeed);
                                    },
                                  );
                                }
                              },
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
              const SizedBox(height: 16),
              Padding(
      padding: const EdgeInsets.fromLTRB(20, 12, 20, 16),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [ 
          TextButton(
            onPressed: () {onCancel(currentSpeed); Navigator.of(context).pop();},
            child: const Text('Cancel'),
          ),
          const SizedBox(width: 8),
          FilledButton(
            onPressed: (){onConfirm(currentSpeed);Navigator.of(context).pop();},
            child: const Text('Confirm'),
          ),
        ],
      ),
    ),
            ],
          );
        },
      );
    },
  );
}

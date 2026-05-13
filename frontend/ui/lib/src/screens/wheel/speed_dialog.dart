import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

enum SpeedMode {
  constant,
  acp,
}

List<String> speedSliderValues() {
  // Pick speeds relevant to randonneuring.
  /*
  https://www.randonneursmondiaux.org/files/LRM_Event_Regulations_2023.pdf
  https://rusa.org/pages/acp-brevet-control-times-calculator
  https://www.audax-club-parisien.com/organisation/brm-monde/#reglement-BRM
  https://www.audax-club-parisien.com/download/plages_horaires_brm_10_FR.xls
  */
  return [
    "5",
    "10",
    "11.428",
    "12.5",
    "13.333",
    "15",
    "18.0",
    "20",
    "25",
    "28",
  ];
}

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
  required Function(String) onSpeedChanged,
}) {
  SpeedMode initialMode = parseSpeedMode(speed);
  String initialConstantSpeed = initialMode == SpeedMode.constant ? speed : "15";
  
  showDialog(
    context: context,
    builder: (BuildContext context) {
      SpeedMode currentMode = initialMode;
      String constantSpeed = initialConstantSpeed;
      TextEditingController textController = TextEditingController(text: constantSpeed);
      
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
                    String newSpeed = speedModeToString(value, constantSpeed);
                    debugPrint("mode: $currentMode speed: $newSpeed");
                    onSpeedChanged(newSpeed);
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
                      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
                      child: Row(
                        children: [
                          Expanded(
                            child: TextField(
                              controller: textController,
                              enabled: currentMode == SpeedMode.constant,
                              keyboardType: const TextInputType.numberWithOptions(decimal: true),
                              inputFormatters: [
                                FilteringTextInputFormatter.allow(RegExp(r'^\d*\.?\d{0,1}')),
                              ],
                              decoration: const InputDecoration(
                                labelText: 'Speed (km/h)',
                                border: OutlineInputBorder(),
                                suffixText: 'km/h',
                              ),
                              onChanged: (value) {
                                if (value.isNotEmpty && double.tryParse(value) != null) {
                                  constantSpeed = value;
                                  String newSpeed = speedModeToString(currentMode, constantSpeed);
                                  onSpeedChanged(newSpeed);
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
                padding: const EdgeInsets.symmetric(horizontal: 16.0),
                child: ElevatedButton(
                  onPressed: () {
                    Navigator.of(context).pop();
                  },
                  child: const Text('OK', textAlign: TextAlign.center),
                ),
              ),
            ],
          );
        },
      );
    },
  );
}

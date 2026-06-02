import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:wpx/src/utils/utils.dart';
import 'package:wpx/src/widgets/small.dart';

enum SpeedMode { kmh, acp, lrm }

// Pick speeds relevant to randonneuring.
/*
  https://www.randonneursmondiaux.org/files/LRM_Event_Regulations_2023.pdf
  https://rusa.org/pages/acp-brevet-control-times-calculator
  https://www.audax-club-parisien.com/organisation/brm-monde/#reglement-BRM
  https://www.audax-club-parisien.com/download/plages_horaires_brm_10_FR.xls
*/

SpeedMode parseSpeedMode(String speed) {
  if (speed.toUpperCase().contains("ACP")) {
    return SpeedMode.acp;
  }
  if (speed.toUpperCase().contains("LRM")) {
    return SpeedMode.lrm;
  }
  return SpeedMode.kmh;
}

String readKMHSpec(String spec) {
  List<String> parts = spec.split('-');
  if (parts.length > 1) {
    String kmh = parts[1];
    double? ret = double.tryParse(kmh);
    if (ret != null) {
      return kmh;
    }
  }
  return "";
}

String? makeKMHSpec(String text) {
  double? kmh = double.tryParse(text);
  if (kmh == null) {
    return null;
  }
  return "KMH-$kmh";
}

String? selectSpec(List<String> specs, SpeedMode mode) {
  for (String spec in specs) {
    if (mode == SpeedMode.kmh && spec.contains("KMH")) {
      return spec;
    }
    if (mode == SpeedMode.acp && spec.contains("ACP")) {
      return spec;
    }
    if (mode == SpeedMode.lrm && spec.contains("LRM")) {
      return spec;
    }
  }
  return null;
}

class Link {
  final String text;
  final String url;

  Link({required this.text, required this.url});
}

String docsURL(String path) {
  String hostname = "www.julien5.dev";
  // hostname = "www.localhost";
  String root = "https://$hostname/blog/wpx/docs";
  return "$root/$path";
}

Link prettySpeedHeader(String spec) {
  if (parseSpeedMode(spec) == SpeedMode.acp) {
    // input = "ACP-300-20.0";
    List<String> parts = spec.split('-');
    if (parts.length > 2) {
      String km = parts[1];
      String hours = parts[2];
      return Link(
        text: "ACP: $km km, $hours h",
        url: docsURL("UI.html#acp-rules"),
      );
    }
  }
  if (parseSpeedMode(spec) == SpeedMode.lrm) {
    List<String> parts = spec.split('-');
    if (parts.length > 1) {
      String kmh = parts[1];
      return Link(text: "LRM: $kmh kmh", url: docsURL("UI.html#lrm-rules"));
    }
  }
  if (parseSpeedMode(spec) == SpeedMode.kmh) {
    return Link(
      text: "Overall average speed",
      url: docsURL("UI.html#speed--cutoff-times"),
    );
  }
  return Link(
    text: "[$spec]",
    url: "https://www.localhost/blog/wpx/docs/UI.html",
  );
}

String prettySpeed(String spec) {
  if (parseSpeedMode(spec) == SpeedMode.acp) {
    // input = "ACP-300-20.0";
    List<String> parts = spec.split('-');
    if (parts.length > 2) {
      String km = parts[1];
      String hours = parts[2];
      return "ACP: $km/$hours";
    }
  }
  if (parseSpeedMode(spec) == SpeedMode.lrm) {
    List<String> parts = spec.split('-');
    if (parts.length > 1) {
      String kmh = parts[1];
      return "LRM: $kmh kmh";
    }
  }
  if (parseSpeedMode(spec) == SpeedMode.kmh) {
    List<String> parts = spec.split('-');
    if (parts.length > 1) {
      String kmh = parts[1];
      double? d = double.tryParse(kmh);
      if (d != null) {
        return "${formatKmh(d, 3)} kmh";
      }
    }
  }
  return "[$spec]";
}

void openSpeedDialog({
  required BuildContext context,
  required String speed,
  required List<String> allowedSpeeds,
  required String? initialConstantSpeed,
  required Function(String) onSpeedChanged,
  required Function(String) onConfirm,
  required Function(String) onCancel,
}) {
  SpeedMode currentMode = parseSpeedMode(speed);
  String currentSpeed = speed;
  debugPrint("initial:$initialConstantSpeed");
  TextEditingController textController = TextEditingController(
    text:
        initialConstantSpeed != null
            ? readKMHSpec(initialConstantSpeed)
            : "15.0",
  );

  void onChanged(String kmh) {
    if (currentMode != SpeedMode.kmh) {
      return;
    }
    double? currentValue = double.tryParse(kmh);
    assert(currentValue != null);
    if (currentValue == null) {
      return;
    }
    double newValue = currentValue.clamp(1.0, 100.0);
    String? spec = makeKMHSpec(newValue.toString());
    assert(spec != null);
    if (spec != null) {
      currentSpeed = spec;
    }
  }

  void adjustSpeed(double delta) {
    if (currentMode != SpeedMode.kmh) {
      return;
    }
    double? currentValue = double.tryParse(textController.text);
    if (currentValue == null) {
      return;
    }
    double newValue = (currentValue + delta).clamp(1.0, 100.0);
    // Round to 1 decimal place
    newValue = (newValue * 10).round() / 10;

    textController.text = newValue.toString();
    onChanged(textController.text);
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
          List<Widget> widgets = [];
          for (String spec in allowedSpeeds) {
            if (parseSpeedMode(spec) == SpeedMode.kmh) {
              (Widget, Widget) kmhWidgets = kmhTile(
                spec,
                currentSpeed,
                adjustSpeed,
                onChanged,
                textFieldFocusNode,
                textController,
              );
              widgets.add(kmhWidgets.$1);
              widgets.add(kmhWidgets.$2);
            }
            if (parseSpeedMode(spec) == SpeedMode.acp) {
              (Widget, Widget) acpWidgets = acpTile(spec, currentMode);
              widgets.add(acpWidgets.$1);
              widgets.add(acpWidgets.$2);
            }
            if (parseSpeedMode(spec) == SpeedMode.lrm) {
              (Widget, Widget) acpWidgets = acpTile(spec, currentMode);
              widgets.add(acpWidgets.$1);
              widgets.add(acpWidgets.$2);
            }
          }
          return StandardDialog(
            sections: [
              // Header
              DialogHeader(
                label: 'SELECT SPEED',
                title: null,
                url: docsURL("UI.html#speed--cutoff-times"),
              ),
              // Mode section
              Padding(
                padding: DialogStyles.contentPadding,
                child: RadioGroup<SpeedMode>(
                  groupValue: currentMode,
                  onChanged: (SpeedMode? speedMode) {
                    if (speedMode != null) {
                      setDialogState(() {
                        currentMode = speedMode;
                      });
                      if (speedMode == SpeedMode.kmh) {
                        String? spec = makeKMHSpec(textController.text);
                        if (spec != null) {
                          currentSpeed = spec;
                        } else {
                          debugPrint("bad input text: ${textController.text}");
                          assert(false);
                        }
                      } else {
                        String? selected = selectSpec(allowedSpeeds, speedMode);
                        if (selected != null) {
                          currentSpeed = selected;
                        } else {
                          debugPrint("bad $allowedSpeeds $speedMode");
                          assert(false);
                        }
                      }
                    }
                  },
                  child: Column(children: widgets),
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

(Widget, Widget) kmhTile(
  String spec,
  String currentSpeed,
  void Function(double) adjustSpeed,
  void Function(String) onChanged,
  FocusNode textFieldFocusNode,
  TextEditingController textController,
) {
  Widget header = RadioListTile<SpeedMode>(
    title: Text(prettySpeedHeader(spec).text),
    value: SpeedMode.kmh,
    controlAffinity: ListTileControlAffinity.leading,
  );

  Widget body = Padding(
    padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
    child: Row(
      children: [
        Expanded(
          child: Listener(
            onPointerSignal: (event) {
              if (event is PointerScrollEvent &&
                  parseSpeedMode(currentSpeed) == SpeedMode.kmh) {
                // Scroll up (negative delta) increases speed
                // Scroll down (positive delta) decreases speed
                double delta = event.scrollDelta.dy > 0 ? -1 : 1;
                adjustSpeed(delta);
              }
            },
            child: TextField(
              focusNode: textFieldFocusNode,
              controller: textController,
              enabled: parseSpeedMode(currentSpeed) == SpeedMode.kmh,
              keyboardType: const TextInputType.numberWithOptions(
                decimal: true,
              ),
              inputFormatters: [
                FilteringTextInputFormatter.allow(RegExp(r'^\d*\.?\d{0,3}')),
              ],
              decoration: const InputDecoration(
                labelText: 'Speed (km/h)',
                border: OutlineInputBorder(),
                suffixText: 'km/h',
              ),
              onChanged: onChanged,
            ),
          ),
        ),
      ],
    ),
  );
  return (header, body);
}

(Widget, Widget) acpTile(String spec, SpeedMode currentMode) {
  Widget header = RadioListTile<SpeedMode>(
    title: Text(prettySpeedHeader(spec).text),
    value: parseSpeedMode(spec),
    controlAffinity: ListTileControlAffinity.leading,
  );
  Widget body = Padding(
    padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
    child: Row(
      children: [
        Expanded(
          child: InputDecorator(
            decoration: InputDecoration(
              labelText: "Warning",
              border: OutlineInputBorder(),
              suffixText: '',
              enabled: currentMode == parseSpeedMode(spec),
            ),
            child: Text(
              'Very unofficial implementation.',
              style: TextStyle(
                color:
                    currentMode == parseSpeedMode(spec)
                        ? Colors.black
                        : Colors.grey,
              ),
            ),
          ),
        ),
      ],
    ),
  );
  return (header, body);
}

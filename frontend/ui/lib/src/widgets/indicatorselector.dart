import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';

class ElevationIndicatorGroup extends StatefulWidget {
  const ElevationIndicatorGroup({super.key});

  @override
  State<ElevationIndicatorGroup> createState() =>
      _ElevationIndicatorGroupState();
}

class _ElevationIndicatorGroupState extends State<ElevationIndicatorGroup> {
  String? selectedValue = "none";
  static const String none = "none";
  static const String ticks = "ticks";
  static const String percent = "percent";

  @override
  void initState() {
    super.initState();
    readModel();
  }

  void readModel() {
    ParameterModel parameterModel = Provider.of<ParameterModel>(context, listen: false);
    Parameters p = parameterModel.parameters();
    for (var indicator in p.profileOptions.elevationIndicators) {
      if (indicator == ProfileIndication.numericSlope) {
        selectedValue = percent;
      }
      if (indicator == ProfileIndication.gainTicks) {
        selectedValue = ticks;
      }
      if (indicator == ProfileIndication.none) {
        selectedValue = none;
      }
    }
  }

  void updateModel() {
    ParameterModel parameters = Provider.of<ParameterModel>(context, listen: false);
    if (selectedValue == none) {
      parameters.setProfileIndication(ProfileIndication.none);
    } else if (selectedValue == percent) {
      parameters.setProfileIndication(ProfileIndication.numericSlope);
    } else if (selectedValue == ticks) {
      parameters.setProfileIndication(ProfileIndication.gainTicks);
    }
  }

  void onChanged(String? newValue) {
    setState(() {
      selectedValue = newValue;
    });
    updateModel();
  }

  @override
  Widget build(BuildContext context) {
    ListTileControlAffinity left = ListTileControlAffinity.trailing;
    return RadioGroup<String>(
      groupValue: selectedValue,
      onChanged: onChanged,
      child: Column(
        mainAxisSize: MainAxisSize.min, // Center the column vertically
        children: [
          RadioListTile<String>(
            title: const Text("Elevation ticks"),
            value: ticks,
            controlAffinity: left,
          ),
          RadioListTile<String>(
            title: const Text("Average slope per intervals"),
            value: percent,
            controlAffinity: left,
          ),
          RadioListTile<String>(
            title: const Text("None"),
            value: none,
            controlAffinity: left,
          ),
        ],
      ),
    );
  }
}

class ElevationIndicatorChooser extends StatelessWidget {
  const ElevationIndicatorChooser({super.key});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: 20.0,
        ), // Add margin inside the parent
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 300),
          child: ElevationIndicatorGroup(),
        ),
      ),
    );
  }
}

import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/slidervalues.dart';
import 'package:ui/utils.dart';

enum SelectedParameter { distance, elevation, none }

class UserStepsModel extends ChangeNotifier {
  final SegmentModel segmentModel;
  SelectedParameter selectedParameter = SelectedParameter.none;
  final Map<SelectedParameter, List<double>> _sliderValues = {};
  final Map<SelectedParameter, double> _selectedValue = {};
  UserStepsModel({required this.segmentModel}) {
    _sliderValues[SelectedParameter.distance] = fromKm([5, 10, 15, 20, 25]);
    _sliderValues[SelectedParameter.elevation] = [
      10,
      25,
      50,
      100,
      200,
      250,
      300,
      400,
      500,
    ];
    _selectedValue[SelectedParameter.elevation] =
        _sliderValues[SelectedParameter.elevation]![1];
    _selectedValue[SelectedParameter.distance] =
        _sliderValues[SelectedParameter.distance]![1];
    selectedParameter = readBackendParameter();
    double? value = readBackendValue();
    if (value != null) {
      _selectedValue[selectedParameter] = value;
    } else {
      assert(selectedParameter == SelectedParameter.none);
    }
  }

  SelectedParameter readBackendParameter() {
    UserStepsOptions p = segmentModel.userStepsOptions();
    if (p.stepDistance == null && p.stepElevationGain == null) {
      return SelectedParameter.none;
    }
    if (p.stepDistance != null) {
      return SelectedParameter.distance;
    }
    return SelectedParameter.elevation;
  }

  double? readBackendValue() {
    UserStepsOptions p = segmentModel.userStepsOptions();
    if (p.stepDistance == null && p.stepElevationGain == null) {
      return null;
    }
    if (p.stepDistance != null) {
      return p.stepDistance;
    }
    return p.stepElevationGain;
  }

  SliderValues? sliderValues(SelectedParameter p) {
    SliderValues ret = SliderValues();
    assert(_sliderValues.containsKey(p));
    assert(_selectedValue.containsKey(p));
    ret.init(_sliderValues[p]!, _selectedValue[p]!);
    return ret;
  }

  double currentValue(SelectedParameter p) {
    assert(_selectedValue.containsKey(p));
    return _selectedValue[p]!;
  }

  void updateValue(SelectedParameter p, double value) {
    _selectedValue[p] = value;
    notifyListeners();
    sendToBackend(p);
  }

  /*
   * Changing the root model has no effect because the segments are cached
   * in SegmentsScreen. User steps handling must be fixed.
   */
  void sendToBackend(SelectedParameter parameter) {
    segmentModel.setUserStepsOptions(makeUserStepsOptions(parameter));
  }

  void sendParameterToBackend(SelectedParameter parameter) {
    selectedParameter = parameter;
    notifyListeners();
    sendToBackend(parameter);
  }

  UserStepsOptions makeUserStepsOptions(SelectedParameter parameter) {
    if (parameter == SelectedParameter.none) {
      return UserStepsOptions(stepDistance: null, stepElevationGain: null);
    }
    double current = currentValue(parameter);
    if (parameter == SelectedParameter.distance) {
      return UserStepsOptions(
        stepDistance: current.toDouble(),
        stepElevationGain: null,
      );
    }
    assert(parameter == SelectedParameter.elevation);
    return UserStepsOptions(
      stepDistance: null,
      stepElevationGain: current.toDouble(),
    );
  }
}

List<double> toKm(List<double> list) {
  List<double> ret = list;
  for (int k = 0; k < list.length; ++k) {
    ret[k] = list[k] * 1000;
  }
  return ret;
}

class UserStepsSlider extends StatelessWidget {
  final SelectedParameter widgetParameter;
  final bool enabled;
  const UserStepsSlider({
    super.key,
    required this.widgetParameter,
    required this.enabled,
  });

  void onChanged(UserStepsModel model, double value) {
    model.updateValue(widgetParameter, value);
  }

  String formatLabel(UserStepsModel model, double value) {
    if (widgetParameter == SelectedParameter.elevation) {
      return "${(value).toInt()} m";
    }
    if (widgetParameter == SelectedParameter.distance) {
      return "${(value).toInt() / 1000} km";
    }
    return "$value ??";
  }

  @override
  Widget build(BuildContext context) {
    var model = Provider.of<UserStepsModel>(context);
    var values = model.sliderValues(widgetParameter);
    if (values == null) {
      return const Text('not set yet');
    }
    return SliderValuesWidget(
      values: values,
      onChanged: (value) {
        return onChanged(model, value);
      },
      formatLabel: (value) {
        return formatLabel(model, value);
      },
      enabled: enabled,
    );
  }
}

class UserStepsSliderConsumer extends StatefulWidget {
  const UserStepsSliderConsumer({super.key});

  @override
  State<UserStepsSliderConsumer> createState() =>
      _UserStepsSliderConsumerState();
}

typedef MenuEntry = DropdownMenuEntry<String>;

class _UserStepsSliderConsumerState extends State<UserStepsSliderConsumer> {
  SelectedParameter selectedParameter = SelectedParameter.none;

  void onSelected(SelectedParameter? value) {
    UserStepsModel model = Provider.of<UserStepsModel>(context, listen: false);
    if (value != null) {
      selectedParameter = value;
    } else {
      selectedParameter = SelectedParameter.none;
    }
    developer.log("selected $value");
    model.sendParameterToBackend(selectedParameter);
  }

  @override
  Widget build(BuildContext context) {
    UserStepsModel model = Provider.of<UserStepsModel>(context);
    developer.log("rebuild with selected ${model.selectedParameter}");
    Widget distanceSlider = UserStepsSlider(
      widgetParameter: SelectedParameter.distance,
      enabled: model.selectedParameter == SelectedParameter.distance,
    );
    Widget elevationSlider = UserStepsSlider(
      widgetParameter: SelectedParameter.elevation,
      enabled: model.selectedParameter == SelectedParameter.elevation,
    );
    final ListTileControlAffinity side = ListTileControlAffinity.leading;

    double km = model.currentValue(SelectedParameter.distance) / 1000;
    Text kmtext = Text(
      "${km.toStringAsFixed(0)} km",
      style: TextStyle(
        color:
            selectedParameter == SelectedParameter.distance
                ? Colors.black
                : Colors.grey,
      ),
    );

    double hm = model.currentValue(SelectedParameter.elevation);
    Text hmtext = Text(
      "${hm.toStringAsFixed(0)} m",
      style: TextStyle(
        color:
            selectedParameter == SelectedParameter.elevation
                ? Colors.black
                : Colors.grey,
      ),
    );

    return RadioGroup<SelectedParameter>(
      groupValue: selectedParameter,
      onChanged: onSelected,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          RadioListTile<SelectedParameter>(
            title: Text("None", textAlign: TextAlign.start),
            value: SelectedParameter.none,
            controlAffinity: side,
          ),
          RadioListTile<SelectedParameter>(
            title: Row(children: [distanceSlider, kmtext]),
            value: SelectedParameter.distance,
            controlAffinity: side,
          ),
          RadioListTile<SelectedParameter>(
            title: Row(children: [elevationSlider, hmtext]),
            value: SelectedParameter.elevation,
            controlAffinity: side,
          ),
        ],
      ),
    );
  }
}

class UserStepsSliderWidget extends StatelessWidget {
  const UserStepsSliderWidget({super.key});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 500),
        child: UserStepsSliderConsumer(),
      ),
    );
  }
}

class UserStepsSliderProvider extends StatelessWidget {
  const UserStepsSliderProvider({super.key});

  @override
  Widget build(BuildContext context) {
    SegmentModel model = Provider.of<SegmentModel>(context);
    return ChangeNotifierProvider(
      create: (ctx) => UserStepsModel(segmentModel: model),
      builder: (context, child) {
        return UserStepsSliderWidget();
      },
    );
  }
}

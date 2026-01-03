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
  UserStepsOptions? currentOptions;

  final Map<SelectedParameter, List<double>> _sliderValues = {};
  final Map<SelectedParameter, double> _selectedValue = {};

  static SelectedParameter parameter(UserStepsOptions options) {
    if (options.stepDistance != null) {
      return SelectedParameter.distance;
    }

    if (options.stepElevationGain != null) {
      return SelectedParameter.elevation;
    }

    return SelectedParameter.none;
  }

  static double? value(UserStepsOptions options) {
    if (options.stepDistance == null && options.stepElevationGain == null) {
      return null;
    }
    if (options.stepDistance != null) {
      return options.stepDistance;
    }
    return options.stepElevationGain;
  }

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

    // defaults
    _selectedValue[SelectedParameter.distance] =
        _sliderValues[SelectedParameter.distance]![1];
    _selectedValue[SelectedParameter.elevation] =
        _sliderValues[SelectedParameter.elevation]![1];

    currentOptions = segmentModel.userStepsOptions();
    assert(currentOptions != null);
    var v = value(currentOptions!);
    if (v != null) {
      _selectedValue[parameter(currentOptions!)] = v;
    }
  }

  SliderValues? sliderValues(SelectedParameter p) {
    SliderValues ret = SliderValues();
    assert(_sliderValues.containsKey(p));
    assert(_selectedValue.containsKey(p));
    ret.init(_sliderValues[p]!, _selectedValue[p]!);
    return ret;
  }

  SelectedParameter getSelectedParameter() {
    return parameter(currentOptions!);
  }

  double getCurrentValue(SelectedParameter p) {
    return _selectedValue[p]!;
  }

  void updateValue(double value) {
    SelectedParameter p = parameter(currentOptions!);
    _updateOptions(p, value);
    _selectedValue[p] = value;
  }

  void updateParameter(SelectedParameter p) {
    double? v = _selectedValue[p];
    _updateOptions(p, v);
  }

  void updateParameterValue(SelectedParameter parameter, double value) {
    _selectedValue[parameter] = value;
    _updateOptions(parameter, value);
  }

  /*
   * Changing the root model has no effect because the segments are cached
   * in SegmentsScreen. User steps handling must be fixed.
   */
  void _sendParameterToBackend() {
    notifyListeners();
    segmentModel.setUserStepsOptions(currentOptions!);
  }

  void _updateOptions(SelectedParameter parameter, double? value) {
    if (parameter == SelectedParameter.none) {
      currentOptions = UserStepsOptions(
        stepDistance: null,
        stepElevationGain: null,
        gpxNameFormat: currentOptions!.gpxNameFormat,
      );
    } else if (parameter == SelectedParameter.distance) {
      currentOptions = UserStepsOptions(
        stepDistance: value!,
        stepElevationGain: null,
        gpxNameFormat: currentOptions!.gpxNameFormat,
      );
    } else {
      assert(parameter == SelectedParameter.elevation);
      currentOptions = UserStepsOptions(
        stepDistance: null,
        stepElevationGain: value!,
        gpxNameFormat: currentOptions!.gpxNameFormat,
      );
    }
    _sendParameterToBackend();
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
    model.updateParameterValue(widgetParameter, value);
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
    model.updateParameter(selectedParameter);
  }

  @override
  Widget build(BuildContext context) {
    UserStepsModel model = Provider.of<UserStepsModel>(context);
    selectedParameter = model.getSelectedParameter();
    developer.log("rebuild with selected $selectedParameter");
    Widget distanceSlider = UserStepsSlider(
      widgetParameter: SelectedParameter.distance,
      enabled: selectedParameter == SelectedParameter.distance,
    );
    Widget elevationSlider = UserStepsSlider(
      widgetParameter: SelectedParameter.elevation,
      enabled: selectedParameter == SelectedParameter.elevation,
    );
    final ListTileControlAffinity side = ListTileControlAffinity.leading;

    double km = model.getCurrentValue(SelectedParameter.distance) / 1000;
    Text kmtext = Text(
      "one point every ${km.toStringAsFixed(0)} km",
      textAlign: TextAlign.start, // Added to flush text to the left
      style: TextStyle(
        fontSize: 13,
        color:
            selectedParameter == SelectedParameter.distance
                ? Colors.black
                : Colors.grey,
      ),
    );

    double hm = model.getCurrentValue(SelectedParameter.elevation);
    Text hmtext = Text(
      "one point every ${hm.toStringAsFixed(0)} m elevation gain",
      textAlign: TextAlign.start, // Added to flush text to the left
      style: TextStyle(
        fontSize: 13,
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
        mainAxisAlignment: MainAxisAlignment.start,
        children: [
          RadioListTile<SelectedParameter>(
            title: Row(
              children: [
                SizedBox(width: 25),
                Text("None", textAlign: TextAlign.start),
              ],
            ),
            value: SelectedParameter.none,
            controlAffinity: side,
          ),
          SizedBox(height: 30),
          RadioListTile<SelectedParameter>(
            title: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                distanceSlider,
                Row(children: [SizedBox(width: 25), kmtext]),
              ],
            ),
            value: SelectedParameter.distance,
            controlAffinity: side,
          ),
          SizedBox(height: 30),
          RadioListTile<SelectedParameter>(
            title: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                elevationSlider,
                Row(children: [SizedBox(width: 25), hmtext]),
              ],
            ),
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

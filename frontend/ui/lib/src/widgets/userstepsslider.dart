import 'dart:collection';
import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/slidervalues.dart';
import 'package:ui/utils.dart';

enum SelectedParameter { distance, elevation }

class UserStepsModel extends ChangeNotifier {
  final SegmentModel segmentModel;
  SelectedParameter? selectedParameter;
  final Map<SelectedParameter, List<double>> _sliderValues = {};
  final Map<SelectedParameter, double> _selectedValues = {};
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
    _selectedValues[SelectedParameter.elevation] =
        _sliderValues[SelectedParameter.elevation]![1];
    _selectedValues[SelectedParameter.distance] =
        _sliderValues[SelectedParameter.distance]![1];
    selectedParameter = _readRootSelected();
    if (selectedParameter != null) {
      double? value = _readRootValue();
      assert(value != null);
      _selectedValues[selectedParameter!] = value!;
    }
  }

  SelectedParameter? _readRootSelected() {
    UserStepsOptions p = segmentModel.userStepsOptions();
    if (p.stepDistance == null && p.stepElevationGain == null) {
      return null;
    }
    if (p.stepDistance != null) {
      return SelectedParameter.distance;
    }
    return SelectedParameter.elevation;
  }

  double? _readRootValue() {
    UserStepsOptions p = segmentModel.userStepsOptions();
    if (p.stepDistance == null && p.stepElevationGain == null) {
      return null;
    }
    if (p.stepDistance != null) {
      return p.stepDistance;
    }
    return p.stepElevationGain;
  }

  SliderValues? sliderValues() {
    if (selectedParameter == null) {
      return null;
    }
    SliderValues ret = SliderValues();
    ret.init(
      _sliderValues[selectedParameter]!,
      _selectedValues[selectedParameter]!,
    );
    return ret;
  }

  double? currentValue() {
    if (!_selectedValues.containsKey(selectedParameter)) {
      return null;
    }
    return _selectedValues[selectedParameter];
  }

  void updateValue(double value) {
    assert(selectedParameter != null);
    _selectedValues[selectedParameter!] = value;
    notifyListeners();
    _updateBackend();
  }

  /*
   * Changing the root model has no effect because the segments are cached
   * in SegmentsScreen. User steps handling must be fixed.
   */
  void _updateBackend() {
    segmentModel.setUserStepsOptions(makeUserStepsOptions());
  }

  void updateParameter(SelectedParameter? key) {
    selectedParameter = key;
    notifyListeners();
    _updateBackend();
  }

  UserStepsOptions makeUserStepsOptions() {
    double? current = currentValue();
    if (current == null) {
      return UserStepsOptions(stepDistance: null, stepElevationGain: null);
    }
    assert(selectedParameter != null);
    if (selectedParameter == SelectedParameter.distance) {
      return UserStepsOptions(
        stepDistance: current.toDouble(),
        stepElevationGain: null,
      );
    }
    assert(selectedParameter == SelectedParameter.elevation);
    return UserStepsOptions(
      stepDistance: null,
      stepElevationGain: current.toDouble(),
    );
  }

  SegmentStatistics statistics() {
    return segmentModel.statistics();
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
  const UserStepsSlider({super.key});

  void onChanged(UserStepsModel model, double value) {
    model.updateValue(value);
  }

  String formatLabel(UserStepsModel model, double value) {
    if (model.selectedParameter == SelectedParameter.elevation) {
      return "${(value).toInt()} m";
    }
    if (model.selectedParameter == SelectedParameter.distance) {
      return "${(value).toInt() / 1000} km";
    }
    return "$value ??";
  }

  @override
  Widget build(BuildContext context) {
    var model = Provider.of<UserStepsModel>(context);
    var values = model.sliderValues();
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
  static const List<String> list = <String>["none", 'km', 'hm'];
  static final List<MenuEntry> menuEntries = UnmodifiableListView<MenuEntry>(
    list.map<MenuEntry>((String name) => MenuEntry(value: name, label: name)),
  );

  void onSelected(String? value) {
    UserStepsModel model = Provider.of<UserStepsModel>(context, listen: false);
    developer.log("selected $value");
    SelectedParameter? newMode = fromString(value);
    model.updateParameter(newMode);
  }

  String string(SelectedParameter? param) {
    if (param == null) {
      return "none";
    }
    if (param == SelectedParameter.distance) {
      return "km";
    }
    return "hm";
  }

  SelectedParameter? fromString(String? value) {
    SelectedParameter? newMode;
    if (value == "km") {
      newMode = SelectedParameter.distance;
    } else if (value == "hm") {
      newMode = SelectedParameter.elevation;
    }
    return newMode;
  }

  @override
  Widget build(BuildContext context) {
    UserStepsModel model = Provider.of<UserStepsModel>(context);
    developer.log("rebuild with selected ${model.selectedParameter}");
    Widget slider = UserStepsSlider();
    DropdownMenu<String> dropbox = DropdownMenu<String>(
      initialSelection: string(model.selectedParameter),
      onSelected: onSelected,
      dropdownMenuEntries: menuEntries,
    );
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: 20.0,
        ), // Add margin inside the parent
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 400),
          child: Row(children: [slider, dropbox]),
        ),
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
        return UserStepsSliderConsumer();
      },
    );
  }
}

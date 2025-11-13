import 'dart:collection';
import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/screens/settings/slidervalues.dart';
import 'package:ui/utils.dart';

enum SelectedParameter { distance, elevation }

class UserStepsModel extends ChangeNotifier {
  final RootModel rootModel;
  SelectedParameter? selectedParameter;
  final Map<SelectedParameter, List<double>> _sliderValues = {};
  final Map<SelectedParameter, double> _selectedValues = {};
  UserStepsModel({required this.rootModel}) {
    Parameters params = rootModel.parameters();
    SegmentStatistics stats = rootModel.statistics();
    _sliderValues[SelectedParameter.distance] = sensitiveDistanceSteps(params);
    _sliderValues[SelectedParameter.elevation] = sensitiveElevationSteps(
      stats,
      params,
    );
    _selectedValues[SelectedParameter.elevation] =
        _sliderValues[SelectedParameter.elevation]![1];
    _selectedValues[SelectedParameter.distance] =
        _sliderValues[SelectedParameter.distance]![1];
    selectedParameter = _readRootSelected();
    if (selectedParameter != null) {
      double? value = _readRootValue();
      assert(value!=null);
      _selectedValues[selectedParameter!] = value!;
    }
  }

  SelectedParameter? _readRootSelected() {
    ProfileOptions p = parameters().profileOptions;
    if (p.stepDistance == null && p.stepElevationGain == null) {
      return null;
    }
    if (p.stepDistance != null) {
      return SelectedParameter.distance;
    }
    return SelectedParameter.elevation;
  }

  double? _readRootValue() {
    ProfileOptions p = parameters().profileOptions;
    if (p.stepDistance == null && p.stepElevationGain == null) {
      return null;
    }
    if (p.stepDistance != null) {
      return p.stepDistance;
    }
    return p.stepElevationGain;
  }

  Parameters parameters() {
    return rootModel.parameters();
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

  void _updateBackend() {
    Parameters? p = newParameters();
    rootModel.setParameters(p);
  }

  void updateParameter(SelectedParameter? key) {
    selectedParameter = key;
    notifyListeners();
    _updateBackend();
  }

  ProfileOptions _makeProfileOptions(ProfileOptions old) {
    double? current = currentValue();
    if (current == null) {
      return ProfileOptions(
        elevationIndicators: old.elevationIndicators,
        maxAreaRatio: old.maxAreaRatio,
        stepDistance: null,
        stepElevationGain: null,
        size: old.size,
      );
    }
    assert(selectedParameter != null);
    if (selectedParameter == SelectedParameter.distance) {
      return ProfileOptions(
        elevationIndicators: old.elevationIndicators,
        maxAreaRatio: old.maxAreaRatio,
        stepDistance: current.toDouble(),
        stepElevationGain: null,
        size: old.size
      );
    }
    assert(selectedParameter == SelectedParameter.elevation);
    return ProfileOptions(
      elevationIndicators: old.elevationIndicators,
      maxAreaRatio: old.maxAreaRatio,
      stepDistance: null,
      stepElevationGain: current.toDouble(),
      size: old.size
    );
  }

  Parameters newParameters() {
    Parameters oldParameters = rootModel.parameters();
    ProfileOptions old = oldParameters.profileOptions;
    ProfileOptions newp = _makeProfileOptions(old);
    return bridge.Parameters(
      speed: oldParameters.speed,
      startTime: oldParameters.startTime,
      segmentLength: oldParameters.segmentLength,
      segmentOverlap: oldParameters.segmentOverlap,
      smoothWidth: oldParameters.smoothWidth,
      profileOptions: newp,
      mapOptions: oldParameters.mapOptions,
      debug: oldParameters.debug,
    );
  }

  SegmentStatistics statistics() {
    return rootModel.statistics();
  }
}

List<double> toKm(List<double> list) {
  List<double> ret = list;
  for (int k = 0; k < list.length; ++k) {
    ret[k] = list[k] * 1000;
  }
  return ret;
}

List<double> sensitiveDistanceSteps(Parameters parameters) {
  double distance = parameters.segmentLength - parameters.segmentOverlap;
  double distanceKm = distance / 1000;
  List<double> values = [2, 5, 10];
  if (distanceKm > 10) {
    values = [5, 10, 15, 20, 25];
  }
  if (distanceKm > 50) {
    values = [10, 15, 20, 25, 30, 50];
  }
  if (distanceKm > 100) {
    values = [10, 20, 25, 50, 100];
  }
  if (distanceKm > 200) {
    values = [20, 25, 50, 60, 75, 100];
  }
  if (distanceKm > 400) {
    values = [40, 50, 75, 100, 150, 200, 300];
  }
  if (distanceKm > 600) {
    values = [50, 60, 75, 100, 150, 200, 300, 400, 500];
  }
  return fromKm(values);
}

List<double> sensitiveElevationSteps(
  SegmentStatistics trackStatistics,
  Parameters parameters,
) {
  int nsegments =
      (trackStatistics.distanceEnd /
              (parameters.segmentLength - parameters.segmentOverlap))
          .ceil();
  double elevation = trackStatistics.elevationGain / nsegments;
  developer.log("total elevation: ${trackStatistics.elevationGain}} m");
  developer.log("elevation per segment: $elevation m");
  List<double> values = [10, 25, 50];
  if (elevation > 100) {
    values = [25, 50, 75, 100];
  }
  if (elevation > 250) {
    values = [50, 75, 100, 150, 200, 250];
  }
  if (elevation > 500) {
    values = [100, 150, 200, 250, 300, 400, 500];
  }
  if (elevation > 1000) {
    values = [200, 250, 300, 400, 500, 1000];
  }
  if (elevation > 2000) {
    values = [500, 750, 1000, 1500, 2000, 2500];
  }
  if (elevation > 5000) {
    values = [1000, 1500, 2000, 2500, 3000, 4000, 5000];
  }
  if (elevation > 10000) {
    values = [2000, 2500, 3000, 4000, 5000, 10000];
  }
  return values;
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
    RootModel root = Provider.of<RootModel>(context);
    return ChangeNotifierProvider(
      create: (ctx) => UserStepsModel(rootModel: root),
      builder: (context, child) {
        return UserStepsSliderConsumer();
      },
    );
  }
}

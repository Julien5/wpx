import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class AreaParameters {
  double mapRatio = 0;
  double profileRatio = 0;
}

class AreaSliderModel extends ChangeNotifier {
  final RootModel rootModel;
  late AreaParameters areaParameters = AreaParameters();
  AreaSliderModel({required this.rootModel}) {
    areaParameters = current();
  }

  Parameters parameters() {
    return rootModel.parameters();
  }

  AreaParameters current() {
    Parameters params = parameters();
    areaParameters.mapRatio = params.mapOptions.maxAreaRatio;
    areaParameters.profileRatio = params.profileOptions.maxAreaRatio;
    return areaParameters;
  }

  void setMapRatio(double v) {
    areaParameters.mapRatio = v;
    updateBackend();
  }

  void setProfileRatio(double v) {
    areaParameters.profileRatio = v;
    updateBackend();
  }

  void updateBackend() {
    Parameters? p = makeParametersForBackend();
    rootModel.setParameters(p);
  }

  Parameters makeParametersForBackend() {
    Parameters oldParameters = rootModel.parameters();
    return bridge.Parameters(
      speed: oldParameters.speed,
      startTime: oldParameters.startTime,
      segmentLength: oldParameters.segmentLength,
      segmentOverlap: oldParameters.segmentOverlap,
      smoothWidth: oldParameters.smoothWidth,
      profileOptions: ProfileOptions(
        elevationIndicators: oldParameters.profileOptions.elevationIndicators,
        maxAreaRatio: areaParameters.profileRatio,
        size: oldParameters.profileOptions.size,
      ),
      mapOptions: MapOptions(
        maxAreaRatio: areaParameters.mapRatio,
        size: oldParameters.mapOptions.size,
      ),
      userStepsOptions: oldParameters.userStepsOptions,
      debug: oldParameters.debug,
      controlGpxNameFormat: oldParameters.controlGpxNameFormat,
    );
  }
}

class AreaSliderConsumer extends StatefulWidget {
  const AreaSliderConsumer({super.key});

  @override
  State<AreaSliderConsumer> createState() => _AreaSliderConsumerState();
}

typedef MenuEntry = DropdownMenuEntry<String>;

class _AreaSliderConsumerState extends State<AreaSliderConsumer> {
  void onSelected(double value) {
    AreaSliderModel model = Provider.of<AreaSliderModel>(
      context,
      listen: false,
    );
    developer.log("selected $value");
    model.setMapRatio(value);
  }

  @override
  Widget build(BuildContext context) {
    AreaSliderModel model = Provider.of<AreaSliderModel>(context);
    developer.log("rebuild with selected ${model.areaParameters}");
    Widget row1 = Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        const Text("Profile label ratio"),
        SizedBox(width: 16),
        Slider(
          value: 5 * model.areaParameters.profileRatio.clamp(0.0, 1.0),
          min: 0.0,
          max: 1.0,
          onChanged: (value) {
            model.setProfileRatio(0.2 * value);
          },
        ),
      ],
    );
    Widget row2 = Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        const Text("Map label ratio"),
        SizedBox(width: 16),
        Slider(
          value: 5 * model.areaParameters.mapRatio.clamp(0.0, 1.0),
          min: 0.0,
          max: 1.0,
          onChanged: (value) {
            model.setMapRatio(0.2 * value);
          },
        ),
      ],
    );
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: 20.0,
        ), // Add margin inside the parent
        child: Column(children: [row1, row2]),
      ),
    );
  }
}

class AreaSlider extends StatelessWidget {
  const AreaSlider({super.key});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    return ChangeNotifierProvider(
      create: (ctx) => AreaSliderModel(rootModel: root),
      builder: (context, child) {
        return AreaSliderConsumer();
      },
    );
  }
}

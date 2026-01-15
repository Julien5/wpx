import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/segmentsgraphicsrow.dart';
import 'package:ui/src/widgets/slidervalues.dart';
import 'package:ui/utils.dart';

class TextWidget extends StatelessWidget {
  const TextWidget({super.key});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    String text = "${root.segments().length} segments";
    return Center(child: Text(text));
  }
}

List<double> segmentLengthSliderValues(double trackLength) {
  double trackLengthKm = trackLength / 1000;
  List<double> values = [2, 5, 10];
  if (trackLengthKm > 10) {
    values = [5, 10, 25, 50];
  }
  if (trackLengthKm > 50) {
    values = [10, 25, 50, 100];
  }
  if (trackLengthKm > 100) {
    values = [25, 50, 100, 150, 200];
  }
  if (trackLengthKm > 200) {
    values = [50, 100, 150, 200, 400];
  }
  if (trackLengthKm > 400) {
    values = [100, 150, 200, 300, 600];
  }
  if (trackLengthKm > 600) {
    values = [100, 150, 200, 300, 600, 1000];
  }
  return fromKm(values);
}

class SliderWidget extends StatelessWidget {
  const SliderWidget({super.key});

  void onValueChanged(BuildContext context, double value) {
    RootModel root = Provider.of<RootModel>(context, listen: false);
    Parameters p = root.parameters();
    ParameterChanger changer = ParameterChanger(init: p);
    changer.changeSegmentLength(value);
    root.setParameters(changer.current());
    developer.log("length:${value / 1000} km");
  }

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    double trackLength = root.statistics().length;
    List<double> values = segmentLengthSliderValues(trackLength);
    Widget w = SliderValuesWidget(
      values: values,
      initIndex: 1,
      formatLabel: (value) => "${(value / 1000).toStringAsFixed(1)} km",
      onValueChanged: (value) => {onValueChanged(context, value)},
      enabled: true,
    );

    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 500),
        child: w,
      ),
    );
  }
}

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext ctx) {
    Widget row = SegmentsGraphicsRow(kinds: allkinds(), height: 200);
    return Scaffold(
      appBar: AppBar(title: const Text('PDF')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            row,
            Divider(height: 5),
            TextWidget(),
            SizedBox(height: 10),
            SliderWidget(),
          ],
        ),
      ),
    );
  }
}

class SettingsScreenProviders extends MultiProvider {
  SettingsScreenProviders({
    super.key,
    required SegmentModel segmentModel,
    required TrackViewsSwitch multiTrackModel,
    required Widget child,
  }) : super(
         providers: [
           ChangeNotifierProvider.value(value: segmentModel),
           ChangeNotifierProvider.value(value: multiTrackModel),
         ],
         child: child,
       );
}

class SettingsProvider extends StatelessWidget {
  final SegmentModel model;
  final TrackViewsSwitch trackViewSwitch;
  const SettingsProvider({
    super.key,
    required this.model,
    required this.trackViewSwitch,
  });

  @override
  Widget build(BuildContext context) {
    return SettingsScreenProviders(
      segmentModel: model,
      multiTrackModel: trackViewSwitch,
      child: SettingsScreen(),
    );
  }
}

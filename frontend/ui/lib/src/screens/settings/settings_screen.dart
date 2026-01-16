import 'dart:developer' as developer;
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/segmentsgraphicsrow.dart';
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

List<int> niceSegmentLengths() {
  List<int> km = [
    10,
    15,
    20,
    25,
    30,
    35,
    40,
    50,
    60,
    75,
    100,
    150,
    200,
    250,
    300,
    400,
    500,
    750,
    1000,
  ];
  return km.map((e) => e * 1000).toList();
}

double niceSegmentLength(double value) {
  for (int p in niceSegmentLengths()) {
    if (p > value) {
      return p.toDouble();
    }
  }
  return 0;
}

int segmentCount(double trackLength, double segmentLength) {
  double segmentOverlap = ((segmentLength * 0.1 / 1.1) / 1000).round() * 1000;
  return (trackLength / (segmentLength - segmentOverlap)).ceil();
}

int projectNumberOfPages(int wanted, double trackLength, Parameters p) {
  double nice = niceSegmentLength(0.5 * trackLength / wanted);
  double segmentOverlap = nice / 10;
  double segmentLength = nice + segmentOverlap;
  int nsegment = segmentCount(trackLength, segmentLength);
  int npages = (nsegment * 0.5).ceil();
  return npages;
}

class SliderWidget extends StatelessWidget {
  const SliderWidget({super.key});

  void onChanged(BuildContext context, double pages) {
    RootModel root = Provider.of<RootModel>(context, listen: false);
    double trackLength = root.statistics().length;
    double nice = niceSegmentLength(0.5 * trackLength / pages);
    double segmentOverlap = nice / 10;
    double segmentLength = nice + segmentOverlap;
    Parameters p = root.parameters();
    ParameterChanger changer = ParameterChanger(init: p);
    changer.changeSegmentLength(segmentLength);
    changer.changeSegmentOverlap(segmentOverlap);
    root.setParameters(changer.current());
    developer.log("wanted:$pages pages");
    int npages = projectNumberOfPages(pages.round(), trackLength, p);
    developer.log("length:${nice / 1000} km => $npages pages");
  }

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    Parameters parameters = root.parameters();
    double trackLength = root.statistics().length;
    double segmentLength = parameters.segmentLength;
    double segmentOverlap = parameters.segmentOverlap;
    assert(segmentOverlap == (segmentLength - segmentOverlap) / 10);
    int nsegment = segmentCount(trackLength, segmentLength);
    int bsegment = root.segments().length;
    assert(nsegment == bsegment);

    int high = projectNumberOfPages(5, trackLength, parameters);

    int lmin = niceSegmentLengths().reduce(min);
    int lmax = niceSegmentLengths().reduce(max);
    int pmax = min(
      high,
      (segmentCount(trackLength, lmin.toDouble()) / 2).ceil(),
    );
    int pmin = (segmentCount(trackLength, lmax.toDouble()) / 2).ceil();

    int p = ((nsegment * 0.5).ceil()).clamp(pmin, pmax);

    developer.log("$pmin $pmax $p $segmentLength");
    return Slider(
      min: pmin.toDouble(),
      max: pmax.toDouble(),
      divisions: pmax - pmin,
      value: p.toDouble(),
      label: "$p pages",
      onChanged: (value) => {onChanged(context, value)},
    );
  }
}

class SettingsWidget extends StatelessWidget {
  const SettingsWidget({super.key});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    List<Segment> segments = root.segments();
    Parameters parameters = root.parameters();

    String km =
        "${((parameters.segmentLength - parameters.segmentOverlap) / 1000).ceil().toString().padLeft(3)} km per segment";
    String segmentCount = "${segments.length.toString().padLeft(2)} segments";
    String pageCount =
        "${(segments.length / 2).ceil().toString().padLeft(2)} pages";
    String countText = "$segmentCount on $pageCount";
    SizedBox placeHolder = SizedBox(
      width: 120, // or 40–56 depending on your design
      child: Container(color: Colors.red),
    );
    // there is a bug with Slider in a Table:
    // https://github.com/flutter/flutter/issues/174133
    return Card(
      elevation: 4, // Add shadow to the card
      margin: const EdgeInsets.fromLTRB(
        30,
        5,
        20,
        10,
      ), // Add margin around the card
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8), // Rounded corners
      ),
      child: Column(
        children: [
          Row(
            children: [
              const Padding(
                padding: EdgeInsets.all(8.0),
                child: Text("Number of pages:"),
              ),
              SizedBox(
                width: 200, // or 40–56 depending on your design
                child: SliderWidget(),
              ),
            ],
          ),
          Row(
            children: [
              placeHolder,
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [Text(km)],
                ),
              ),
            ],
          ),
          Row(
            children: [
              placeHolder,
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [Text(countText)],
                ),
              ),
            ],
          ),
          Row(
            children: [
              placeHolder,
              const Padding(
                padding: EdgeInsets.all(8.0),
                child: Text("(2 segments per page)"),
              ),
            ],
          ),
        ],
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
            SizedBox(height: 10),
            SettingsWidget(),
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

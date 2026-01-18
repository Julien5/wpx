import 'dart:developer' as developer;
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/segmentgraphics.dart';
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
  final VoidCallback? onShowPressed;
  final bool show;
  const SettingsWidget({
    super.key,
    required this.onShowPressed,
    required this.show,
  });

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    List<Segment> segments = root.segments();
    Parameters parameters = root.parameters();
    String segLength = ((parameters.segmentLength - parameters.segmentOverlap) /
            1000)
        .ceil()
        .toString()
        .padLeft(3);
    String pageCount = (segments.length / 2).ceil().toString().padLeft(2);
    IconData showIcon = Icons.arrow_right;
    if (show) {
      showIcon = Icons.arrow_drop_down;
    }
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
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: Text("$pageCount pages"),
              ),
              SizedBox(
                width: 200, // or 40–56 depending on your design
                child: SliderWidget(),
              ),
            ],
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    ElevatedButton.icon(
                      onPressed: onShowPressed,
                      icon: Icon(showIcon, color: Colors.green, size: 30.0),
                      label: Text(
                        "$segLength km per segment",
                        style: TextStyle(fontSize: 12),
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class TopRow extends StatelessWidget {
  const TopRow({super.key});
  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (_) => TrackViewsSwitch(exposed: [TrackData.pages]),
      child: TrackGraphicsRow(kinds: allkinds(), height: 200),
    );
  }
}

class BottomRow extends StatelessWidget {
  const BottomRow({super.key});
  @override
  Widget build(BuildContext context) {
    developer.log("[LocalSegmentGraphics]");
    return ChangeNotifierProvider(
      create:
          (_) => TrackViewsSwitch(exposed: [TrackData.profile, TrackData.map]),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Divider(height: 5),
          SegmentsGraphicsRow(kinds: allkinds(), height: 200),
          Divider(height: 5),
        ],
      ),
    );
  }
}

class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key});

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  bool showBottomWidget = false;
  void onShowPressed() {
    setState(() {
      showBottomWidget = !showBottomWidget;
    });
  }

  @override
  Widget build(BuildContext ctx) {
    return Scaffold(
      appBar: AppBar(title: const Text('PDF')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            TopRow(),
            ConstrainedBox(
              constraints: BoxConstraints(maxWidth: 500),
              child: SettingsWidget(
                show: showBottomWidget,
                onShowPressed: onShowPressed,
              ),
            ),
            if (showBottomWidget) BottomRow(),
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
  const SettingsProvider({super.key, required this.model});

  @override
  Widget build(BuildContext context) {
    return SettingsScreenProviders(
      segmentModel: model,
      multiTrackModel: TrackViewsSwitch(exposed: TrackViewsSwitch.wmp()),
      child: SettingsScreen(),
    );
  }
}

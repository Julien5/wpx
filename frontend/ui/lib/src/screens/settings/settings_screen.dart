import 'dart:async';
import 'dart:developer' as developer;
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/adaptive_layout.dart';
import 'package:ui/src/widgets/segmentgraphics.dart';
import 'package:ui/src/widgets/segmentsgraphicsrow.dart';
import 'package:ui/utils.dart';

// TODO: make this code clean.
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

int maxPageCount(double rawLength) {
  double km = rawLength / 1000;
  debugPrint("km=$km");
  if (km > 1000) {
    return (km / 100).ceil();
  }
  if (km > 100) {
    return (km / 50).ceil();
  }
  return (km / 20).ceil();
}

List<int> niceSegmentLengths(double rawLength) {
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
  ];
  List<int> ret = km.map((e) => e * 1000).toList();
  double hundredk = 100000;
  double up100 = (rawLength / hundredk).ceil() * hundredk;
  ret.add(up100.toInt());
  ret.sort();
  return ret;
}

double niceSegmentLength(double rawLength) {
  for (int p in niceSegmentLengths(rawLength)) {
    if (p > rawLength) {
      return p.toDouble();
    }
  }
  assert(false);
  return 0;
}

int segmentCount(double trackLength, double segmentLength) {
  double segmentOverlap = ((segmentLength * 0.1 / 1.1) / 1000).round() * 1000;
  return (trackLength / (segmentLength - segmentOverlap)).ceil();
}

int projectNumberOfPages(int wanted, double trackLength, Parameters p) {
  double nice = niceSegmentLength(trackLength / wanted);
  double segmentOverlap = nice / 10;
  double segmentLength = nice + segmentOverlap;
  return segmentCount(trackLength, segmentLength);
}

class PagesSliderWidget extends StatefulWidget {
  const PagesSliderWidget({super.key});

  @override
  State<PagesSliderWidget> createState() => _PagesSliderWidgetState();
}

class PageCountInfo {
  int pmin = 1;
  int pmax = 2;
  int npages = 1;
}

class _PagesSliderWidgetState extends State<PagesSliderWidget> {
  Timer? _debounceTimer;
  final PageCountInfo _pages = PageCountInfo();

  void updatePagesInfo(
    double trackLength,
    Parameters parameters,
    int desiredPageCount,
  ) {
    int rawmax = maxPageCount(trackLength);
    int high = projectNumberOfPages(rawmax, trackLength, parameters);
    int lmin = niceSegmentLengths(trackLength).reduce(min);
    int lmax = niceSegmentLengths(trackLength).reduce(max);
    assert(lmin < lmax);
    _pages.pmax = min(high, segmentCount(trackLength, lmin.toDouble()));
    _pages.pmin = segmentCount(trackLength, lmax.toDouble());
    _pages.npages = projectNumberOfPages(
      desiredPageCount,
      trackLength,
      parameters,
    ).clamp(_pages.pmin, _pages.pmax);
  }

  void changePageCount(BuildContext context, int pages) {
    ParameterModel parameters = Provider.of<ParameterModel>(
      context,
      listen: false,
    );
    SegmentModel track = Provider.of<SegmentModel>(context, listen: false);
    double trackLength = track.statistics().length;
    double nice = niceSegmentLength(trackLength / pages);
    double segmentOverlap = nice / 10;
    double segmentLength = nice + segmentOverlap;
    ParameterChanger changer = ParameterChanger(init: parameters.parameters());
    changer.changeSegmentLength(segmentLength);
    changer.changeSegmentOverlap(segmentOverlap);
    parameters.setParameters(changer.current());
    updatePagesInfo(trackLength, parameters.parameters(), pages);
    developer.log("length:${nice / 1000} km => ${_pages.npages} pages");
  }

  void onChanged(BuildContext context, double pages) {
    _debounceTimer?.cancel();
    _debounceTimer = Timer(const Duration(milliseconds: 250), () {
      changePageCount(context, pages.round());
    });
    setState(
      () => _pages.npages = pages.floor().clamp(_pages.pmin, _pages.pmax),
    );
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    SegmentModel track = Provider.of(context, listen: false);
    ParameterModel parameterModel = Provider.of(context, listen: false);
    double trackLength = track.statistics().length;
    updatePagesInfo(trackLength, parameterModel.parameters(), _pages.npages);
  }

  @override
  Widget build(BuildContext context) {
    Provider.of<SegmentModel>(context);
    Provider.of<ParameterModel>(context);
    return Slider(
      min: _pages.pmin.toDouble(),
      max: _pages.pmax.toDouble(),
      divisions: _pages.pmax - _pages.pmin,
      value: _pages.npages.toDouble(),
      label: "${_pages.npages} pages",
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
    SegmentModel segment = Provider.of<SegmentModel>(context);
    ParameterModel parameterModel = Provider.of<ParameterModel>(context);
    List<Segment> segments = segment.backend.segments();
    Parameters parameters = parameterModel.parameters();
    String segLength = ((parameters.segmentLength - parameters.segmentOverlap) /
            1000)
        .ceil()
        .toString()
        .padLeft(3);
    String pageCount = segments.length.toString().padLeft(2);
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
                child: PagesSliderWidget(),
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
                      label: Text("$segLength km per page"),
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
    context.watch<ParameterModel>();
    return ChangeNotifierProvider(
      create: (_) => TrackViewsSwitch(exposed: [TrackData.wheelPages]),
      child: TrackGraphicsRow(kinds: allkinds()),
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
          (_) => TrackViewsSwitch(
            exposed: [TrackData.profile, TrackData.map],
            sizes: {
              TrackData.profile: Size(1000, 300),
              TrackData.map: Size(400, 400),
            },
          ),
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

class SettingsScaffold extends StatefulWidget {
  const SettingsScaffold({super.key});

  @override
  State<SettingsScaffold> createState() => _SettingsScaffoldState();
}

class _SettingsScaffoldState extends State<SettingsScaffold> {
  bool showBottomWidget = false;
  void onShowPressed() {
    debugPrint("PRESSED");
    setState(() {
      showBottomWidget = !showBottomWidget;
    });
  }

  @override
  Widget build(BuildContext ctx) {
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.arrow_back),
          onPressed: () => gotoOverview(ctx),
        ),
        title: const Text('PDF'),
      ),
      body: AdaptiveLayout(
        topRow: TopRow(),
        midChildren: [
          SettingsWidget(show: showBottomWidget, onShowPressed: onShowPressed),
          if (showBottomWidget) BottomRow(),
        ],
      ),
    );
  }
}

class SettingsScreenProviders extends MultiProvider {
  SettingsScreenProviders({
    super.key,
    required TrackViewsSwitch multiTrackModel,
    required Widget child,
  }) : super(
         providers: [ChangeNotifierProvider.value(value: multiTrackModel)],
         child: child,
       );
}

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return SettingsScreenProviders(
      multiTrackModel: TrackViewsSwitch(exposed: TrackViewsSwitch.wmp()),
      child: SettingsScaffold(),
    );
  }
}

import 'dart:async';
import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/models/stackviewscontroller.dart';
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/utils/print.dart';
import 'package:wpx/src/widgets/adaptive_layout.dart';
import 'package:wpx/src/widgets/segmentgraphics.dart';
import 'package:wpx/src/widgets/segmentsgraphicsrow.dart';
import 'package:wpx/src/utils/utils.dart';

class PagesSliderWidget extends StatefulWidget {
  const PagesSliderWidget({super.key});

  @override
  State<PagesSliderWidget> createState() => _PagesSliderWidgetState();
}

class _PagesSliderWidgetState extends State<PagesSliderWidget> {
  Timer? _debounceTimer;
  PageCountInfo? _pages;

  void changePageIndex(BuildContext context, int desiredPageIndex) async {
    ParameterModel parameters = Provider.of<ParameterModel>(
      context,
      listen: false,
    );
    _pages!.setPossiblePageIndex(desiredPageIndex);
    debugPrint("print setPossiblePageIndex: $desiredPageIndex");
    ParameterChanger changer = ParameterChanger(init: parameters.parameters());
    debugPrint("print desiredPageIndex: $desiredPageIndex");
    double length = _pages!.getSegmentLengthWithOverlap();
    double overlap = _pages!.getSegmentOverlap();
    debugPrint("print input length: $length overlap:$overlap");
    changer.changeSegmentLength(length);
    changer.changeSegmentOverlap(overlap);
    parameters.setParameters(changer.current());
    Parameters output = parameters.parameters();
    debugPrint(
      "print output length: ${output.segmentLength} overlap:${output.segmentOverlap}",
    );
    _pages!.setParameters(output.segmentLength, output.segmentOverlap);
  }

  void onChanged(BuildContext context, double index) {
    _debounceTimer?.cancel();
    _debounceTimer = Timer(const Duration(milliseconds: 250), () {
      changePageIndex(context, index.round());
    });

    setState(() {
      _pages!.setPossiblePageIndex(index.round());
    });
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    SegmentModel track = Provider.of(context, listen: false);
    ParameterModel parameterModel = Provider.of(context, listen: false);
    double trackLength = track.statistics().length;
    _pages ??= PageCountInfo(
      trackLength,
      parameterModel.parameters().segmentLength,
    );
    Parameters p = parameterModel.parameters();
    _pages!.setParameters(p.segmentLength, p.segmentOverlap);
  }

  @override
  Widget build(BuildContext context) {
    Provider.of<SegmentModel>(context);
    Provider.of<ParameterModel>(context);

    return Slider(
      min: _pages!.getMinIndex(),
      max: _pages!.getMaxIndex(),
      divisions: _pages!.possiblePageCounts.length - 1,
      value: _pages!.possiblePageIndex().toDouble(),
      label: PageCountInfo.getPagesCountString(_pages!.getSegmentCount()),
      onChanged: (index) => {onChanged(context, index)},
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
    ParameterModel parameterModel = Provider.of<ParameterModel>(context);

    RootModel root = Provider.of(context);
    List<Segment> segments = root.backend.segments();
    Parameters parameters = parameterModel.parameters();
    String segLength = (segmentLengthWithoutOverlap(parameters) / 1000)
        .ceil()
        .toString()
        .padLeft(3);
    String pageCount = PageCountInfo.getPagesCountString(segments.length);
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
                child: Text(pageCount),
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
      create: (_) => StackViewsController(exposed: [RenderFunction.wheelPages]),
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
          (_) => StackViewsController(
            exposed: [RenderFunction.profile, RenderFunction.map],
            sizes: {
              RenderFunction.profile: Size(1000, 300),
              RenderFunction.map: Size(400, 400),
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
      body: MobileScaffoldBody(
        topRow: TopRow(),
        midColumn: MidColumn(
          children: [
            SettingsWidget(
              show: showBottomWidget,
              onShowPressed: onShowPressed,
            ),
            if (showBottomWidget) BottomRow(),
          ],
        ),
        screenFocus: ScreenFocus.settings,
        clients: [RenderFunction.wheelPages],
      ),
    );
  }
}

class SettingsScreenProviders extends MultiProvider {
  SettingsScreenProviders({
    super.key,
    required StackViewsController multiTrackModel,
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
      multiTrackModel: StackViewsController(
        exposed: StackViewsController.wmp(),
      ),
      child: SettingsScaffold(),
    );
  }
}

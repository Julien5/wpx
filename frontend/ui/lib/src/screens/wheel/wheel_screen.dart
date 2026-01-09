import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/controls/controls_screen.dart';
import 'package:ui/src/screens/segments/future_rendering_widget.dart';
import 'package:ui/src/screens/usersteps/usersteps_screen.dart';
import 'package:ui/src/screens/wheel/statistics_widget.dart';

class RendererParameters {
  final Set<InputType> kinds;
  final TrackData trackData;
  const RendererParameters({required this.kinds, required this.trackData});
  ValueKey createKey() {
    final sortedKinds = kinds.map((k) => k.toString()).toList()..sort();
    return ValueKey('${trackData.toString()}|${sortedKinds.join(",")}');
  }
}

class SvgWidget extends StatefulWidget {
  final RendererParameters parameters;
  final Size rendererSize;
  const SvgWidget({
    super.key,
    required this.rendererSize,
    required this.parameters,
  });

  @override
  State<SvgWidget> createState() => _SvgWidgetState();
}

class _SvgWidgetState extends State<SvgWidget> {
  FutureRenderer? renderer;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _initState());
  }

  void _initState() {
    assert(mounted);
    SegmentModel model = Provider.of<SegmentModel>(context, listen: false);
    renderer = model.giveRenderer(
      widget.parameters.kinds,
      widget.parameters.trackData,
    );
    if (widget.parameters.trackData == TrackData.wheel) {
      renderer!.setSize(widget.rendererSize);
    } else {
      renderer!.setSize(widget.rendererSize * 1.5);
    }
    renderer!.addListener(_onRendererChanged);
    _onRendererChanged();
  }

  @override
  void dispose() {
    super.dispose();
    renderer!.removeListener(_onRendererChanged);
  }

  void _onRendererChanged() {
    developer.log("[_onRendererChanged] _onRendererChanged");
    if (!mounted) {
      return;
    }
    assert(renderer != null);
    setState(() {
      if (renderer!.needsStart()) {
        renderer!.start();
      }
    });
  }

  @override
  Widget build(BuildContext ctx) {
    if (renderer == null) {
      return Text("hi");
    }
    return FutureRenderingWidget(future: renderer!, interactive: false);
  }
}

class LayoutWidget extends StatelessWidget {
  final RendererParameters parameters;
  const LayoutWidget({super.key, required this.parameters});

  static LayoutWidget make(Set<InputType> kinds, TrackData trackData) {
    RendererParameters p = RendererParameters(
      kinds: kinds,
      trackData: trackData,
    );
    return LayoutWidget(key: p.createKey(), parameters: p);
  }

  @override
  Widget build(BuildContext ctx) {
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        Size size = constraints.biggest;
        developer.log("ProfileWidget size: $size");
        return SvgWidget(rendererSize: size, parameters: parameters);
      },
    );
  }
}

class WhiteWidget extends StatelessWidget {
  final Color color;
  const WhiteWidget({super.key, required this.color});

  @override
  Widget build(BuildContext ctx) {
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        Size size = constraints.biggest;
        developer.log("WhiteWidget size: $size");
        final double width =
            size.width.isFinite ? size.width : constraints.maxWidth;
        final double height =
            size.height.isFinite ? size.height : constraints.maxHeight;
        return Container(width: width, height: height, color: color);
      },
    );
  }
}

class TrackMultiView extends StatefulWidget {
  final Set<InputType> kinds;
  const TrackMultiView({super.key, required this.kinds});

  @override
  State<TrackMultiView> createState() => _TrackMultiViewState();
}

class _TrackMultiViewState extends State<TrackMultiView> {
  List<Widget> widgets = [Text("loading"), Text("loading"), Text("loading")];

  @override
  void initState() {
    super.initState();
    widgets.clear();
    widgets.add(LayoutWidget.make(widget.kinds, TrackData.profile));
    widgets.add(LayoutWidget.make(widget.kinds, TrackData.map));
    widgets.add(WhiteWidget(key: const ValueKey('white'), color: Colors.white));
    widgets.add(LayoutWidget.make(widget.kinds, TrackData.wheel));
  }

  void onTap() {
    setState(() {
      int end = widgets.length - 1;
      Widget current = widgets[end];
      widgets[end] = widgets[1];
      widgets[1] = widgets[0];
      widgets[0] = current;
    });
  }

  @override
  Widget build(BuildContext ctx) {
    double margin = 8;
    return GestureDetector(
      onTap: onTap,
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxHeight: 200),
        child: Padding(
          padding: EdgeInsetsGeometry.fromLTRB(margin, margin, margin, margin),
          child: SizedBox(
            width: double.infinity,
            height: 200,
            child: Stack(children: widgets),
          ),
        ),
      ),
    );
  }
}

class WheelScreen extends StatelessWidget {
  const WheelScreen({super.key});

  void gotoSettings(BuildContext ctx) {
    Navigator.of(ctx).pushNamed(RouteManager.settingsView);
  }

  Future<void> gotoUserSteps(BuildContext ctx) async {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    await Navigator.push(
      ctx,
      MaterialPageRoute(
        builder: (context) => UserStepsProvider(model: model.copy()),
      ),
    );
    model.notify();
  }

  Future<void> gotoControls(BuildContext ctx) async {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    Navigator.push(
      ctx,
      MaterialPageRoute(
        builder: (context) => ControlsProvider(model: model.copy()),
      ),
    );
    model.notify();
  }

  @override
  Widget build(BuildContext ctx) {
    Widget pdfButton = ElevatedButton(
      onPressed: () => gotoSettings(ctx),
      child: const Text("PDF"),
    );

    Widget controlsButtons = ElevatedButton(
      onPressed: () => gotoControls(ctx),
      child: const Text("Controls"),
    );

    Widget userStepsButton = ElevatedButton(
      onPressed: () => gotoUserSteps(ctx),
      child: const Text("Pacing"),
    );

    Card statisticsCard = Card(
      elevation: 4, // Add shadow to the card
      margin: const EdgeInsets.all(1), // Add margin around the card
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8), // Rounded corners
      ),
      child: Padding(
        padding: const EdgeInsets.all(16), // Add padding inside the card
        child: StatisticsWidget(),
      ),
    );

    Widget vspace = SizedBox(height: 50);
    return Scaffold(
      appBar: AppBar(title: const Text('Overview')),
      body: Center(
        child: Container(
          constraints: const BoxConstraints(maxWidth: 500),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              TrackMultiView(kinds: allkinds()),
              Expanded(child: vspace),
              statisticsCard,
              Expanded(child: vspace),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [controlsButtons, userStepsButton, pdfButton],
              ),
              vspace,
            ],
          ),
        ),
      ),
    );
  }
}

class WheelProvider extends StatelessWidget {
  const WheelProvider({super.key});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    Bridge bridge = root.getBridge();
    assert(bridge.isLoaded());
    return ChangeNotifierProvider(
      create: (ctx) => SegmentModel(bridge, root.trackSegment()),
      builder: (context, child) {
        return WheelScreen();
      },
    );
  }
}

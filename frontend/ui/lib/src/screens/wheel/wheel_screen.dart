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

class SvgWidget extends StatefulWidget {
  final Set<InputType> kinds;
  final TrackData trackData;
  final Size rendererSize;
  const SvgWidget({
    super.key,
    required this.rendererSize,
    required this.kinds,
    required this.trackData,
  });

  @override
  State<SvgWidget> createState() => _SvgWidgetState();
}

class _SvgWidgetState extends State<SvgWidget> {
  FutureRenderer? renderer;
  @override
  void initState() {
    super.initState();
  }

  @override
  void dispose() {
    renderer?.removeListener(_onRendererChanged);
    super.dispose();
  }

  void _onRendererChanged() {
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

  FutureRenderer makeRenderer(SegmentModel model) {
    return model.giveRenderer(widget.kinds, widget.trackData);
  }

  @override
  Widget build(BuildContext ctx) {
    if (renderer == null || renderer!.getSize() != widget.rendererSize) {
      developer.log("[REMAKE MODEL] $renderer");
      renderer?.removeListener(_onRendererChanged);
      SegmentModel model = Provider.of<SegmentModel>(ctx);
      renderer = makeRenderer(model);
      renderer!.setSize(widget.rendererSize);
      renderer!.addListener(_onRendererChanged);
      assert(renderer!.getSize() == widget.rendererSize);
    }
    return FutureRenderingWidget(future: renderer!, interactive: false);
  }
}

class ProfileWidget extends StatelessWidget {
  final Set<InputType> kinds;
  const ProfileWidget({super.key, required this.kinds});

  @override
  Widget build(BuildContext ctx) {
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        Size size = constraints.biggest * 1.5;
        developer.log("ProfileWidget size: $size");
        return SvgWidget(
          rendererSize: size,
          kinds: kinds,
          trackData: TrackData.profile,
        );
      },
    );
  }
}

class MapWidget extends StatelessWidget {
  final Set<InputType> kinds;
  const MapWidget({super.key, required this.kinds});

  @override
  Widget build(BuildContext ctx) {
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        Size size = constraints.biggest * 1.5;
        developer.log("ProfileWidget size: $size");
        return SvgWidget(
          rendererSize: size,
          kinds: kinds,
          trackData: TrackData.map,
        );
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

class WheelWidget extends StatelessWidget {
  final Set<InputType> kinds;
  const WheelWidget({super.key, required this.kinds});

  @override
  Widget build(BuildContext ctx) {
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        Size size = constraints.biggest;
        developer.log("ProfileWidget size: $size");
        return SvgWidget(
          rendererSize: size,
          kinds: kinds,
          trackData: TrackData.wheel,
        );
      },
    );
  }
}

class StackWidget extends StatefulWidget {
  final Set<InputType> kinds;
  const StackWidget({super.key, required this.kinds});

  @override
  State<StackWidget> createState() => _StackWidgetState();
}

class _StackWidgetState extends State<StackWidget> {
  List<Widget> widgets = [Text("loading"), Text("loading"), Text("loading")];
  int i = 0;

  @override
  void initState() {
    super.initState();
    widgets.clear();
    widgets.add(
      ProfileWidget(key: const ValueKey('wheel'), kinds: widget.kinds),
    );
    widgets.add(MapWidget(key: const ValueKey('map'), kinds: widget.kinds));
    widgets.add(WhiteWidget(key: const ValueKey('white'), color: Colors.white));
    widgets.add(
      WheelWidget(key: const ValueKey('profile'), kinds: widget.kinds),
    );
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

  void gotoControls(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    Navigator.push(
      ctx,
      MaterialPageRoute(builder: (context) => ControlsProvider(model: model)),
    );
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
              StackWidget(kinds: allkinds()),
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

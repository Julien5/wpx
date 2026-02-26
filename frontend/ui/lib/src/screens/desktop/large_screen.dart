import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/wheel/statistics_widget.dart';
import 'package:ui/src/widgets/trackview.dart';
import 'package:ui/src/widgets/userstepsslider.dart';

class GraphicsPadding extends StatelessWidget {
  final Widget child;

  const GraphicsPadding({super.key, required this.child});
  @override
  Widget build(BuildContext ctx) {
    return Padding(padding: EdgeInsetsGeometry.all(20), child: child);
  }
}

class LeftColumn extends StatelessWidget {
  final double width;
  const LeftColumn({super.key, required this.width});

  @override
  Widget build(BuildContext context) {
    Widget div = Divider(color: Colors.lightBlue, thickness: 1, height: 1);
    List<Widget> leftChildren = [
      div,
      Padding(
        padding: const EdgeInsets.all(15),
        child: OverviewWidget(
          onPacingPointPressed: null,
          onControlsPointPressed: () {},
          onPDFPressed: null,
        ),
      ),
      div,
      Padding(
        padding: const EdgeInsets.all(15),
        child: Card(elevation: 4, child: UserStepsSliderProvider()),
      ),
      div,
    ];
    return ConstrainedBox(
      constraints: BoxConstraints(minWidth: width, maxWidth: width),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        children: leftChildren,
      ),
    );
  }
}

class RightColumn extends StatelessWidget {
  final double width;
  const RightColumn({super.key, required this.width});

  @override
  Widget build(BuildContext context) {
    Widget mwrow = Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        //Expanded(child: Container(color: Colors.blue)),
        Expanded(
          child: GraphicsPadding(
            child: TrackView(
              rendererParameters: RendererParameters(
                kinds: allkinds(),
                trackData: TrackData.map,
              ),
            ),
          ),
        ),
      ],
    );

    List<Widget> rightChildren = [
      ConstrainedBox(
        constraints: BoxConstraints(minHeight: 275, maxHeight: 275),
        child: ProfilePadding(
          child: TrackView(
            rendererParameters: RendererParameters(
              kinds: allkinds(),
              trackData: TrackData.profile,
            ),
          ),
        ),
      ),
      Expanded(child: mwrow),
    ];

    Widget rightCol = Expanded(
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: width),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisAlignment: MainAxisAlignment.end,
          children: rightChildren,
        ),
      ),
    );
    return rightCol;
  }
}

class ProfilePadding extends StatelessWidget {
  final Widget child;

  const ProfilePadding({super.key, required this.child});
  @override
  Widget build(BuildContext ctx) {
    return Padding(
      padding: EdgeInsetsGeometry.fromLTRB(5, 10, 10, 10),
      child: child,
    );
  }
}

class _LargeScaffold extends StatelessWidget {
  @override
  Widget build(BuildContext ctx) {
    ScreenConfiguration screen = Provider.of<ScreenConfiguration>(ctx);
    Widget div = VerticalDivider(
      color: Colors.lightBlue,
      thickness: 1,
      width: 1, // This is the horizontal space the widget occupies
    );
    return Scaffold(
      appBar: AppBar(title: const Text('Overview')),
      body: Row(
        children: [
          div,
          LeftColumn(width: 450),
          div,
          RightColumn(width: screen.width - 500),
        ],
      ),
    );
  }
}

class _LargeScreenProviders extends MultiProvider {
  final SegmentModel segmentModel;
  final List<TrackData> trackData;
  final Kinds kinds;
  _LargeScreenProviders({
    required this.segmentModel,
    required this.trackData,
    required this.kinds,
    required Widget child,
  }) : super(
         providers: [
           ChangeNotifierProvider(
             create: (_) => TrackViewsSwitch(exposed: TrackViewsSwitch.wmp()),
           ),
           ChangeNotifierProvider(
             create:
                 (_) => FutureRenderer(
                   bridge: segmentModel.backend,
                   segment: segmentModel.segment,
                   trackData: trackData,
                   kinds: kinds,
                 ),
           ),
         ],
         child: child,
       );
}

class LargeScreen extends StatelessWidget {
  const LargeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    SegmentModel segmentModel = Provider.of<SegmentModel>(context);
    context.watch<SegmentModel>();
    return _LargeScreenProviders(
      segmentModel: segmentModel,
      trackData: [TrackData.map, TrackData.profile],
      kinds: allkinds(),
      child: _LargeScaffold(),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/settings/settings_screen.dart';
import 'package:ui/src/screens/wheel/statistics_widget.dart';
import 'package:ui/src/widgets/small.dart';
import 'package:ui/src/widgets/trackview.dart';
import 'package:ui/src/widgets/userstepsslider.dart';
import 'package:ui/utils.dart';

class GraphicsPadding extends StatelessWidget {
  final Widget child;

  const GraphicsPadding({super.key, required this.child});
  @override
  Widget build(BuildContext ctx) {
    return Padding(padding: EdgeInsetsGeometry.all(20), child: child);
  }
}

class UserStepsCard extends StatefulWidget {
  final ExpansibleController controller;
  final void Function(BuildContext, ScreenFocus, bool) onExpansionChanged;
  const UserStepsCard({
    super.key,
    required this.controller,
    required this.onExpansionChanged,
  });

  @override
  State<UserStepsCard> createState() => _UserStepsCardState();
}

class _UserStepsCardState extends State<UserStepsCard> {
  @override
  Widget build(BuildContext context) {
    ParameterModel parameterModel = Provider.of<ParameterModel>(context);
    debugPrint("REBUILD PACING CARD");

    String pacingPointsText = getPacingPointText(parameterModel.parameters());
    Widget tile = ExpansionTile(
      title: Row(
        children: [
          SmallText(text: "Pacing points"),
          SmallText(text: pacingPointsText),
        ],
      ),
      controller: widget.controller,
      onExpansionChanged:
          (expanded) => widget.onExpansionChanged(
            context,
            ScreenFocus.usersteps,
            expanded,
          ),
      children: <Widget>[
        Builder(
          builder: (BuildContext context) {
            return UserStepsSliderProvider();
          },
        ),
      ],
    );
    return Card(elevation: 4, child: tile);
  }
}

class PDFCard extends StatefulWidget {
  final ExpansibleController controller;
  final void Function(BuildContext, ScreenFocus, bool) onExpansionChanged;
  const PDFCard({
    super.key,
    required this.controller,
    required this.onExpansionChanged,
  });

  @override
  State<PDFCard> createState() => _PDFCardState();
}

class _PDFCardState extends State<PDFCard> {
  @override
  Widget build(BuildContext context) {
    SegmentModel segment = Provider.of<SegmentModel>(context, listen: false);
    Provider.of<ParameterModel>(context);
    List<Segment> segments = segment.backend.segments();
    String pageCount = segments.length.toString().padLeft(2);

    Widget tile = ExpansionTile(
      title: Row(
        children: [SmallText(text: "PDF"), SmallText(text: "$pageCount pages")],
      ),
      controller: widget.controller,
      onExpansionChanged:
          (expanded) => widget.onExpansionChanged(
            context,
            ScreenFocus.settings,
            expanded,
          ),
      children: <Widget>[
        Row(
          children: [SmallText(text: "Number of pages:"), PagesSliderWidget()],
        ),
      ],
    );
    return Card(elevation: 4, child: tile);
  }
}

class SidePanel extends StatefulWidget {
  final double width;
  const SidePanel({super.key, required this.width});

  @override
  State<SidePanel> createState() => _SidePanelState();
}

class _SidePanelState extends State<SidePanel> {
  final ExpansibleController _userStepsController = ExpansibleController();
  final ExpansibleController _pdfController = ExpansibleController();

  void onExpansionChanged(BuildContext context, ScreenFocus f, bool expanded) {
    debugPrint("focus:$f expanded:$expanded");
    FociModel fociModel = Provider.of<FociModel>(context, listen: false);

    if (expanded) {
      fociModel.setFocus(f);
      // When one tile expands, collapse the other
      if (f == ScreenFocus.usersteps) {
        _pdfController.collapse();
      } else if (f == ScreenFocus.settings) {
        _userStepsController.collapse();
      }
    } else {
      fociModel.removeFocus(f);
    }
  }

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
        child: UserStepsCard(
          controller: _userStepsController,
          onExpansionChanged: onExpansionChanged,
        ),
      ),
      div,
      Padding(
        padding: const EdgeInsets.all(15),
        child: PDFCard(
          controller: _pdfController,
          onExpansionChanged: onExpansionChanged,
        ),
      ),
      div,
    ];
    return ConstrainedBox(
      constraints: BoxConstraints(
        minWidth: widget.width,
        maxWidth: widget.width,
      ),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        children: leftChildren,
      ),
    );
  }
}

class MainPanel extends StatelessWidget {
  final double width;
  const MainPanel({super.key, required this.width});

  @override
  Widget build(BuildContext context) {
    Widget mwrow = Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
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
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.home),
          onPressed: () => gotoHome(ctx),
        ),
        title: const Text('Overview'),
      ),
      body: Row(
        children: [
          div,
          SidePanel(width: 480),
          div,
          MainPanel(width: screen.width - 500),
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

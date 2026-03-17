import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';
import 'package:ui/src/screens/desktop/central_panel.dart';
import 'package:ui/src/widgets/waypoints_table_widget.dart';
import 'package:ui/src/utils/utils.dart';

class CentralWidget extends StatelessWidget {
  final double width;
  const CentralWidget({super.key, required this.width});

  @override
  Widget build(BuildContext context) {
    Provider.of<ParameterModel>(context);
    SegmentModel segmentModel = Provider.of<SegmentModel>(context);
    RootModel root = Provider.of<RootModel>(context);
    Segment segment = segmentModel.segment;
    SegmentStatistics stat = root.backend.segmentStatistics(segment: segment);
    debugPrint("CENTRAL segment: ${segment.id()}: ${statisticsString(stat)}");

    FutureRenderer renderer = Provider.of<FutureRenderer>(context);
    RenderOutput? renderOutput = renderer.renderOutput(RenderFunction.profile);
    Widget table = Text("no waypoints");
    if (renderOutput != null) {
      WaypointContainer container = WaypointContainer.create(
        waypoints: renderOutput.waypoints,
      );
      List<Waypoint> waypoints = decimate(
        segment: segment,
        waypoints: container,
        n: BigInt.from(15),
      );
      table = DesktopTable(waypoints: waypoints);
    }

    Widget bottom = Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(
          child: GraphicsPadding(
            child: TrackView(trackData: TrackData.map, svgSize: Size(400, 400)),
          ),
        ),
        Expanded(child: table),
      ],
    );

    List<Widget> children = [
      ConstrainedBox(
        constraints: BoxConstraints(minHeight: 275, maxHeight: 275),
        child: ProfilePadding(
          child: TrackView(
            trackData: TrackData.profile,
            svgSize: Size(1000, 300),
          ),
        ),
      ),
      Expanded(child: bottom),
    ];

    return ConstrainedBox(
      constraints: BoxConstraints(maxWidth: width),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisAlignment: MainAxisAlignment.end,
        children: children,
      ),
    );
  }
}

class CentralPanelPDF extends StatefulWidget {
  final double width;
  const CentralPanelPDF({super.key, required this.width});

  @override
  State<CentralPanelPDF> createState() => _CentralPanelPDFState();
}

class _CentralPanelPDFState extends State<CentralPanelPDF>
    with TickerProviderStateMixin {
  TabController? _tabController;
  TabBar? _tabBar;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    context.watch<ParameterModel>();
    RootModel root = Provider.of(context, listen: false);
    _tabController = TabController(
      length: root.backend.segments().length,
      vsync: this,
    );
    List<Tab> tabs = [];
    for (int n = 0; n < _tabController!.length; ++n) {
      tabs.add(Tab(text: "${n + 1}"));
    }
    _tabBar = TabBar(controller: _tabController, tabs: tabs);
    assert(_tabController != null);
  }

  @override
  void dispose() {
    _tabController?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    Widget bar = _tabBar!;
    Widget view = CentralPanelTabView(
      tabController: _tabController!,
      width: widget.width,
      clients: [TrackData.map, TrackData.profile],
      kinds: allkinds(),
      screenFocus: ScreenFocus.settings,
      child: CentralWidget(width: widget.width),
    );
    return Column(children: [Expanded(child: view), bar]);
  }
}

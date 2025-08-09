import 'dart:developer' as developer;
import 'dart:math';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/future_rendering_widget.dart';
import 'package:ui/src/hardlegend.dart';
import 'package:ui/src/minisvg.dart';
import 'package:ui/src/waypoints_widget.dart';
import 'package:visibility_detector/visibility_detector.dart';

class Legend extends StatelessWidget {
  const Legend({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<TrackRenderer>(
      builder: (context, trackRenderer, child) {
        SegmentsProvider model = Provider.of<SegmentsProvider>(context);
        var size = Size(1000, 285);
        String svg = model.renderSegmentYAxis(trackRenderer.segment, size);
        return Stack(children: [MiniSvgWidget(svg: svg, size: size)]);
      },
    );
  }
}

class SegmentScrollView extends StatelessWidget {
  const SegmentScrollView({super.key});

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Stack(children: <Widget>[TrackConsumer(), WaypointsConsumer()]),
    );
  }
}

class SegmentStack extends StatelessWidget {
  const SegmentStack({super.key});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        var scrollView = SegmentScrollView();
        var box = SizedBox(
          height: 285,
          child: Stack(
            children: [
              Positioned.fill(child: scrollView),
              if (constraints.maxWidth < 1000)
                Positioned(
                  left: 0,
                  top: 0,
                  bottom: 0,
                  child: SizedBox(width: 50, child: Legend()),
                ),
            ],
          ),
        );
        return ConstrainedBox(
          constraints: const BoxConstraints(
            maxWidth: 1000, // Constrain the width to a maximum of 1000 pixels
          ),
          child: box,
        );
      },
    );
  }
}

class SegmentView extends StatelessWidget {
  const SegmentView({super.key});

  @override
  Widget build(BuildContext context) {
    final ScrollController scrollController = ScrollController();

    scrollController.addListener(() {
      double headerHeight = 56;
      double scrollOffset = max(scrollController.offset - headerHeight, 0);
      developer.log("offset: $scrollController.offset");
      double rowHeight = 25; // Assuming each row has a height of 25
      int firstVisibleRow = (scrollOffset / rowHeight).floor();
      int lastVisibleRow =
          ((scrollOffset +
                      scrollController.position.viewportDimension -
                      headerHeight) /
                  rowHeight)
              .floor();

      developer.log("Visible rows: $firstVisibleRow to $lastVisibleRow");
    });

    var table = SingleChildScrollView(
      controller: scrollController, // Attach the ScrollController here
      scrollDirection: Axis.vertical,
      child: WayPointsConsumer(),
    );

    var stack = SegmentStack();

    return Column(
      children: [
        stack,
        const Divider(
          height: 1, // Thickness of the divider
          color: Colors.grey, // Light stroke color
        ),
        Expanded(child: table),
      ],
    );
  }
}

class TrackConsumer extends StatelessWidget {
  const TrackConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<TrackRenderer>(
      builder: (context, trackRenderer, child) {
        return FutureRenderingWidget(future: trackRenderer);
      },
    );
  }
}

class WaypointsConsumer extends StatefulWidget {
  const WaypointsConsumer({super.key});

  @override
  State<WaypointsConsumer> createState() => _WaypointsConsumerState();
}

class _WaypointsConsumerState extends State<WaypointsConsumer> {
  double visibility = 0;

  void onVisibilityChanged(VisibilityInfo info) {
    if (!mounted) {
      return;
    }
    WaypointsRenderer wp = Provider.of<WaypointsRenderer>(
      context,
      listen: false,
    );
    developer.log(
      "[waypoint consumer] id:${wp.id()} vis:${info.visibleFraction}",
    );
    wp.updateVisibility(info.visibleFraction);
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<WaypointsRenderer>(
      builder: (context, waypointsRenderer, child) {
        // It would be more accurate to check visibility with a scroll controller
        // at the list view level. Because "Callbacks are not fired immediately
        // on visibility changes."
        return VisibilityDetector(
          key: Key('id:${waypointsRenderer.id()}'),
          onVisibilityChanged: onVisibilityChanged,
          child: FutureRenderingWidget(future: waypointsRenderer),
        );
      },
    );
  }
}

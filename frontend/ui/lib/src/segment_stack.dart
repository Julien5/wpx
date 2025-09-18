import 'dart:developer' as developer;
import 'dart:math';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/future_rendering_widget.dart';
import 'package:ui/src/futurerenderer.dart';
//import 'package:ui/src/hardlegend.dart';
import 'package:ui/src/waypoints_widget.dart';

class SegmentScrollView extends StatelessWidget {
  const SegmentScrollView({super.key});

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Stack(children: <Widget>[ProfileConsumer()]),
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
                  child: SizedBox(width: 50, child: YAxisConsumer()),
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

  Widget rowWithMap(Widget table) {
    var hspace = const Expanded(child: SizedBox(width: 10));
    /*var _map = Container(
      // Another fixed-width child.
      color: const Color(0xfff01f01),
      width: 300, // Changed to width
      height: 300,
      alignment: Alignment.center,
      child: const Text('Fixed Width Content 3'),
    );
    */
    var map = MapConsumer();
    var row = Expanded(
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [hspace, map, hspace, table, hspace],
      ),
    );
    return row;
  }

  Widget rowWithoutMap(Widget table) {
    var row = Expanded(
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [table],
      ),
    );
    return row;
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
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
        Widget? row;
        if (constraints.maxWidth > 1000) {
          row = rowWithMap(table);
        } else {
          row = rowWithoutMap(table);
        }
        var hline = const Divider(
          height: 1, // Thickness of the divider
          color: Colors.grey, // Light stroke color
        );
        return Column(children: [stack, hline, row]);
      },
    );
  }
}

class MapConsumer extends StatelessWidget {
  const MapConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<MapRenderer>(
      builder: (context, mapRenderer, child) {
        return FutureRenderingWidget(future: mapRenderer);
      },
    );
  }
}

class ProfileConsumer extends StatelessWidget {
  const ProfileConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<ProfileRenderer>(
      builder: (context, pRenderer, child) {
        // It would be more accurate to check visibility with a scroll controller
        // at the list view level. Because "Callbacks are not fired immediately
        // on visibility changes."
        return FutureRenderingWidget(future: pRenderer);
      },
    );
  }
}

class YAxisConsumer extends StatelessWidget {
  const YAxisConsumer({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<YAxisRenderer>(
      builder: (context, yRenderer, child) {
        return FutureRenderingWidget(future: yRenderer);
      },
    );
  }
}

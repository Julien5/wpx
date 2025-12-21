import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/waypoints_table_widget.dart';

class ControlsTableWidget extends StatelessWidget {
  const ControlsTableWidget({super.key});

  void setShortFormat(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    model.setControlGpxNameFormat("NAME[3]-TIME[%H:%M]");
  }

  void setMediumFormat(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    model.setControlGpxNameFormat("NAME[3]-TIME[%H:%M]-SLOPE[4.1%]");
  }

  void setLongFormat(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    model.setControlGpxNameFormat("NAME[*]-TIME[%H:%M]-SLOPE[4.1%]");
  }

  @override
  Widget build(BuildContext ctx) {
    Widget shortButton = ElevatedButton(
      onPressed: () => setShortFormat(ctx),
      child: const Text("short"),
    );

    Widget mediumButton = ElevatedButton(
      onPressed: () => setMediumFormat(ctx),
      child: const Text("medium"),
    );

    Widget longButton = ElevatedButton(
      onPressed: () => setLongFormat(ctx),
      child: const Text("long"),
    );

    Widget buttons = Card(
      elevation: 4, // Add shadow to the card
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8), // Rounded corners
      ),
      child: Padding(
        padding: const EdgeInsets.all(50),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            shortButton,
            SizedBox(width: 10),
            mediumButton,
            SizedBox(width: 10),
            longButton,
          ],
        ),
      ),
    );

    Widget column = Column(
      mainAxisAlignment: MainAxisAlignment.start,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Divider(),
        buttons,
        SizedBox(height: 30),
        Expanded(
          child: Card(
            elevation: 4, // Add shadow to the card
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(8), // Rounded corners
            ),
            child: WaypointsTableWidget(kind: InputType.control),
          ),
        ),
        Divider(),
        SizedBox(height: 30),
      ],
    );

    return Scaffold(
      appBar: AppBar(title: const Text('Control Points Table')),
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(
            maxWidth: 400,
          ), // Set max width to 400px
          child: column,
        ),
      ),
    );
  }
}

class ControlsTableScreen extends StatelessWidget {
  final SegmentModel model;
  const ControlsTableScreen({super.key, required this.model});

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider.value(
      value: model,
      builder: (innercontext, child) {
        return ControlsTableWidget();
      },
    );
  }
}

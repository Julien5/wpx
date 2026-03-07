import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/widgets/small.dart';
import 'model.dart';

class _Padding extends StatelessWidget {
  final Widget child;

  const _Padding({required this.child});
  @override
  Widget build(BuildContext ctx) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: 8.0, horizontal: 16.0),
      child: child,
    );
  }
}

class PartsCard extends StatelessWidget {
  const PartsCard({super.key});

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    if (!model.hasDone(Job.parts)) {
      return Card(elevation: 4, child: _Padding(child: Text("loading ...")));
    }
    List<bridge.TrackPart> parts = model.parts();

    Widget listWidget = SmallText(text: parts[0].name);
    if (parts.length > 1) {
      listWidget = SizedBox(
        height: 200, // Adjust this height as needed
        child: ReorderableListView(
          onReorder: (oldIndex, newIndex) {
            // adjust newIndex if dragging down
            if (oldIndex < newIndex) {
              newIndex -= 1;
            }
            // notify model of the reorder
            model.reorderParts(oldIndex, newIndex);
          },
          children: [
            for (int i = 0; i < parts.length; i++)
              Padding(
                key: ValueKey(i),
                padding: EdgeInsets.symmetric(vertical: 8.0, horizontal: 16.0),
                child: Row(
                  children: [
                    ReorderableDragStartListener(
                      index: i,
                      child: Icon(Icons.drag_handle),
                    ),
                    SizedBox(width: 12),
                    Expanded(child: SmallText(text: parts[i].name)),
                  ],
                ),
              ),
          ],
        ),
      );
    }

    Widget header = _Padding(
      child: Row(children: [SmallText(text: "Segments")]),
    );
    Widget body = listWidget;

    return Card(elevation: 4, child: Column(children: [header, body]));
  }
}

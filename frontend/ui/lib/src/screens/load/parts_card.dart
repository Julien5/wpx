import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/screens/load/eventwidget.dart';
import 'package:wpx/src/widgets/small.dart';
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
    LoadScreenModel model = ctx.watch<LoadScreenModel>();
    debugPrint("running:${model.runningJob()}");
    if (!model.hasDone(Job.parts)) {
      if (model.hasFailed(Job.parts) != null) {
        return Card(
          elevation: 4,
          child: _Padding(child: EventWidget(target: Job.parts)),
        );
      }
      return Card(elevation: 4, child: _Padding(child: Text("loading ...")));
    }

    List<bridge.TrackPart> parts = model.parts();
    bool enabled = parts.length > 1 && model.doneAll();

    void onReorder(int oldIndex, int newIndex) {
      if (!enabled) {
        return;
      }
      // adjust newIndex if dragging down
      if (oldIndex < newIndex) {
        newIndex -= 1;
      }
      // notify model of the reorder
      model.reorderParts(oldIndex, newIndex);
    }

    List<Widget> children = [
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
    ];

    Widget listWidget =
        enabled
            ? ReorderableListView(
              shrinkWrap: true,
              buildDefaultDragHandles: enabled,
              onReorderItem: onReorder,
              children: children,
            )
            : ListView(shrinkWrap: true, children: children);

    Widget header = _Padding(
      child: Row(children: [SmallText(text: "Segments")]),
    );
    Widget body = Flexible(child: listWidget);

    return Card(
      elevation: 4,
      child: Column(mainAxisSize: MainAxisSize.min, children: [header, body]),
    );
  }
}

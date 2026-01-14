import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';

class SideIconButton extends StatelessWidget {
  final VoidCallback? onPressed;
  final TrackData trackData;
  final TrackData selected;
  final double size;
  const SideIconButton({
    super.key,
    required this.selected,
    required this.size,
    required this.trackData,
    this.onPressed,
  });
  final double margin = 8;
  Image icon(TrackData data) {
    String filename = 'icons/png/map.png';
    if (data == TrackData.wheel) {
      filename = 'icons/png/clock.png';
    }
    if (data == TrackData.profile) {
      filename = 'icons/png/profile.png';
    }
    return Image.asset(filename, width: size - margin, height: size - margin);
  }

  @override
  Widget build(BuildContext context) {
    double frameWidth = 1.0;
    if (selected == trackData) {
      frameWidth = 3.0;
    }
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        color: Colors.white,
        border: Border.all(color: Colors.black, width: frameWidth),
        borderRadius: BorderRadius.circular(margin),
      ),
      child: IconButton(
        padding: EdgeInsets.zero,
        constraints: BoxConstraints.tight(Size(size, size)),
        icon: icon(trackData),
        onPressed: onPressed,
      ),
    );
  }
}

class SideIconButtons extends StatelessWidget {
  final void Function(TrackData) onButtonPressed;
  final TrackData selected;
  final double size;

  const SideIconButtons({
    super.key,
    required this.selected,
    required this.size,
    required this.onButtonPressed,
  });

  @override
  Widget build(BuildContext context) {
    const double buttonSize = 30;
    return ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: buttonSize),
      child: Column(
        mainAxisSize: MainAxisSize.max, // Makes Column fill available space
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: [
          SideIconButton(
            selected: selected,
            size: buttonSize,
            trackData: TrackData.wheel,
            onPressed: () => onButtonPressed(TrackData.wheel),
          ),
          SideIconButton(
            selected: selected,
            size: buttonSize,
            trackData: TrackData.map,
            onPressed: () => onButtonPressed(TrackData.map),
          ),
          SideIconButton(
            selected: selected,
            size: buttonSize,
            trackData: TrackData.profile,
            onPressed: () => onButtonPressed(TrackData.profile),
          ),
        ],
      ),
    );
  }
}

class TrackViews extends StatefulWidget {
  final Set<InputType> kinds;
  const TrackViews({super.key, required this.kinds});

  @override
  State<TrackViews> createState() => _TrackViewsState();
}

class _TrackViewsState extends State<TrackViews> {
  Map<TrackData, TrackView> widgets = {};

  @override
  void initState() {
    super.initState();
    assert(widgets.isEmpty);
    for (TrackData data in {
      TrackData.profile,
      TrackData.map,
      TrackData.wheel,
    }) {
      widgets[data] = TrackView.make(widget.kinds, data);
    }
  }

  void onTap() {
    TrackMultiModel model = Provider.of<TrackMultiModel>(
      context,
      listen: false,
    );
    model.cycle();
  }

  TrackData currentTrackData() {
    TrackMultiModel model = Provider.of<TrackMultiModel>(context);
    return model.currentData();
  }

  @override
  Widget build(BuildContext ctx) {
    // Instanciating a Provider.of<Model>(context) (listen=true)
    // is necessary to get rebuild on notifyListeners.
    Provider.of<TrackMultiModel>(context);
    double margin = 8;
    developer.log("rebuild view");
    TrackData currentModelData = currentTrackData();

    // I would like to have `visible = widgets[currentModelData]`
    // but then the widget states are disposed.
    // AI says: In Flutter, when you swap a widget out of the build tree,
    // the previous widget is unmounted and its State object is disposed of.
    // Solution: Stack with Offstaged widgets.
    Widget visible = Stack(
      fit: StackFit.expand, // <--- Add this line
      children:
          widgets.entries.map((entry) {
            return Offstage(
              offstage: entry.key != currentModelData,
              child: entry.value,
            );
          }).toList(),
    );
    return GestureDetector(
      onTap: onTap,
      child: Padding(
        padding: EdgeInsetsGeometry.fromLTRB(margin, margin, margin, margin),
        child: visible,
      ),
    );
  }
}

class TrackMultiModel extends ChangeNotifier {
  TrackData current = TrackData.wheel;
  void cycle() {
    if (current == TrackData.wheel) {
      return changeCurrent(TrackData.map);
    }
    if (current == TrackData.map) {
      return changeCurrent(TrackData.profile);
    }
    if (current == TrackData.profile) {
      return changeCurrent(TrackData.wheel);
    }
  }

  TrackData currentData() {
    return current;
  }

  void changeCurrent(TrackData d) {
    current = d;
    notifyListeners();
  }
}

import 'dart:developer' as developer;
import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class ChooseData extends StatefulWidget {
  const ChooseData({super.key});

  @override
  State<ChooseData> createState() => _ChooseDataState();
}

class FindResult {
  List<int>? bytes;
  String? filename;
  bool demo = false;

  static FindResult makeFilenameResult(String filename) {
    var ret = FindResult();
    ret.filename = filename;
    return ret;
  }

  static FindResult makeBytesResult(List<int> bytes) {
    var ret = FindResult();
    ret.bytes = bytes;
    return ret;
  }

  static FindResult makeDemoResult() {
    var ret = FindResult();
    ret.demo = true;
    return ret;
  }
}

class _ChooseDataState extends State<ChooseData> {
  FindResult? findResult;
  String? errorMessage; // State variable to store the error message

  void chooseGPX(RootModel rootModel) async {
    FilePickerResult? result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ["gpx"],
    );
    if (result == null) {
      return;
    }
    developer.log("result: ${result.count}");
    FindResult? findResult;
    for (var file in result.files) {
      if (!kIsWeb) {
        findResult = FindResult.makeFilenameResult(file.path!);
      } else {
        var bytes = file.bytes!.buffer.asInt8List().toList();
        findResult = FindResult.makeBytesResult(bytes);
      }
      break;
    }
    onDone(rootModel, findResult!);
  }

  Future<void> create(RootModel model, FindResult findResult) async {
    if (findResult.demo) {
      await model.loadDemo(); // Await the async call
    } else if (findResult.bytes != null) {
      await model.loadContent(findResult.bytes!);
    } else if (findResult.filename != null) {
      await model.loadFilename(findResult.filename!);
    } else {
      assert(false);
    }
  }

  void onDone(RootModel model, FindResult findResult) async {
    try {
      await create(model, findResult);
      if (!mounted) {
        return;
      }
      setState(() {
        errorMessage = null; // clears the error message on success
      });
      developer.log("[push]");
      Navigator.of(context).pushNamed(RouteManager.settingsView);
    } catch (e) {
      setState(() {
        errorMessage = makeErrorMessage(e);
      });
    }
  }

  String makeErrorMessage(Object e) {
    if (e is bridge.Error_MissingElevation) {
      //bridge.Error_MissingElevation ev=e;
      var index = e.index;
      return "The track misses elevation data (at index=$index).";
    }
    if (e is bridge.Error_GPXHasNoSegment) {
      return "The GPX file has no segments.";
    }
    if (e is bridge.Error_GPXInvalid) {
      return "The GPX file is malformed.";
    }
    return "An unknown error occurred: ${e.toString()}";
  }

  void chooseDemo(RootModel model) {
    onDone(model, FindResult.makeDemoResult());
  }

  Widget buildFromModel(BuildContext ctx, RootModel rootModel, Widget? child) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          StreamWidget(),
          ElevatedButton(
            onPressed: () => chooseGPX(rootModel),
            child: const Text("Choose GPX file"),
          ),
          if (errorMessage != null) // Conditionally display the error message
            Padding(
              padding: const EdgeInsets.only(top: 10),
              child: Text(
                errorMessage!,
                style: const TextStyle(color: Colors.red),
              ),
            ),
          const SizedBox(height: 20),
          ElevatedButton(
            onPressed: () => chooseDemo(rootModel),
            child: const Text("Demo"),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<RootModel>(
      builder: (context, rootModel, child) {
        return buildFromModel(context, rootModel, child);
      },
    );
  }
}

class StreamWidget extends StatefulWidget {
  const StreamWidget({super.key});

  @override
  State<StreamWidget> createState() => _StreamWidgetState();
}

class _StreamWidgetState extends State<StreamWidget> {
  EventModel? model;
  @override
void initState() {
  super.initState();
  WidgetsBinding.instance.addPostFrameCallback((_) {
    try {
      setState(() {
        model = Provider.of<RootModel>(context, listen: false).eventModel();
      });
    } catch (e) {
      developer.log("Error: RootModel not found in context. Exception: $e");
    }
  });
}

  @override
  Widget build(BuildContext context) {
    if (model == null) {
      return Text("loading..");
    }
    return StreamBuilder<String>(
      stream: model!.stream,
      builder: (context, snap) {
        final error = snap.error;
        String text = "<null>";
        if (error != null) {
          text = error.toString();
          developer.log("error: ${error.toString()}");
        }
        final data = snap.data;
        if (data != null) {
          text = data;
        }
        return Text('text=$text');
      },
    );
  }
}

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext ctx) {
    return ChooseData();
  }
}

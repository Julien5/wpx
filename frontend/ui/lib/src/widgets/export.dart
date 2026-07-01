import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:file_saver/file_saver.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/kindsmodel.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/utils/utils.dart';

enum Type { pdf, gpx, zip }

String fileExtension(Type type) {
  if (type == Type.pdf) {
    return "pdf";
  }
  if (type == Type.zip) {
    return "zip";
  }
  return "gpx";
}

MimeType mimeType(Type type) {
  if (type == Type.pdf) {
    return MimeType.pdf;
  }
  if (type == Type.zip) {
    return MimeType.zip;
  }
  return MimeType.custom;
}

FileType fileType(Type type) {
  if (type == Type.pdf) {
    return FileType.any;
  }
  if (type == Type.zip) {
    return FileType.any;
  }
  return FileType.any;
}

void fileSave(List<int> data) async {
  if (kIsWeb) {
    await FileSaver.instance.saveFile(
      name: "route", // on the web, the extension is set automatically...
      bytes: Uint8List.fromList(data),
      fileExtension: fileExtension(Type.zip),
      mimeType: mimeType(Type.zip),
      customMimeType: fileExtension(Type.zip),
    );
  } else if (Platform.isLinux) {
    var filepath = await FilePicker.platform.saveFile(
      fileName: "route.${fileExtension(Type.zip)}", // .. but not on linux
      type: fileType(Type.zip),
      allowedExtensions: [fileExtension(Type.zip)],
      bytes: Uint8List.fromList(data),
    );
    if (filepath == null) {
      return;
    }
    await Process.run('xdg-open', [filepath]);
  }
}

Future<List<int>> generate(bridge.Bridge backend, Kinds kinds) async {
  var data = await backend.generateZip(kinds: kinds);
  return data;
}

class ExportButton extends StatefulWidget {
  final String text;
  const ExportButton({super.key, required this.text});

  @override
  State<ExportButton> createState() => _ExportButtonState();
}

class _ExportButtonState extends State<ExportButton> {
  bool busy = false;

  void onPressed(bridge.Bridge backend) async {
    if (!mounted) {
      return;
    }
    setState(() {
      busy = true;
    });
    KindsModel kindsModel = context.read();
    var data = await generate(backend, kindsModel.kinds);
    fileSave(data);
    setState(() {
      busy = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    VoidCallback? callback;
    if (!busy) {
      callback = () => onPressed(getBackend(context));
    }
    return ElevatedButton(onPressed: callback, child: Text(widget.text));
  }
}

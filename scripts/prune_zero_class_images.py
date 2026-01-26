#!/usr/bin/env python3
import os
import json
import re

# Example file name
# 20260113_073611.114_4bf13461-5e71-401d-a830-3400a86ca239_0_4.326180935.jpg

file_name_regex = re.compile(r'(\d{8}_\d{6}\.\d+)\_([a-f0-9\-]+)_(\d+)')

def get_info_from_file_name(f):
    m = file_name_regex.search(f)
    if not m:
        return None

    info = {
        'timestamp':     m.group(1),
        'event_id':      m.group(2),
        'capture_index': m.group(3)
    }

    info['id'] = f"{info['event_id']}_{info['capture_index']}"

    return info

def get_image_files(image_dir):
    image_files = []
    detection_files = []

    for root, dirs, files in os.walk(image_dir):
        for f in files:
            if f.endswith(".jpg"):
                image_files.append(os.path.join(root, f))
            elif f.endswith(".detections.json"):
                detection_files.append(os.path.join(root, f))

    return (image_files, detection_files)

def load_json(json_file):
    with open(json_file, 'r') as fh:
        return json.load(fh)

def get_all_image_info(image_dir):
    image_files, detection_files = get_image_files(image_dir)

    images = {}

    for img_file in image_files:
        img_info = get_info_from_file_name(img_file)
        img_id = img_info['id']
        if img_id not in images:
            images[img_id] = {
                'id': img_id,
                'img_file': img_file
            }

    for d_file in detection_files:
        d_file_info = get_info_from_file_name(d_file)
        img_id = d_file_info['id']
        d_info = load_json(d_file)

        if img_id not in images:
            images[img_id] = { 'id': img_id }

        images[img_id]['detection_file'] = d_file
        images[img_id]['detections'] = d_info

    return images

def main():
    image_dir = "var/images"
    image_info = get_all_image_info(image_dir)

    print(f"There are {len(image_info)} image files.")

    for i_ent in image_info.values():
        detection_count = len(i_ent.get('detections', []))
        if detection_count == 0:
            for f_attr in ['img_file', 'detection_file']:
                f = i_ent.get(f_attr)
                if f:
                    print(f"Removing: {f}")
                    os.unlink(f)

main()

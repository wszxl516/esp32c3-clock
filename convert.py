#!/usr/bin/python3
import argparse
import struct

from PIL import Image


def bgr565(args):
    img = Image.open(args.input_file)
    resize = args.resize
    if resize:
        img = img.resize(resize, Image.BILINEAR)
    with open(args.output_file, 'wb') as output_file:
        image = img.convert("RGB").tobytes()
        for i in range(0, image.__len__(), 3):
            r = (image[i + 0] >> 3) & 0x1F
            g = (image[i + 1] >> 2) & 0x3F
            b = (image[i + 2] >> 3) & 0x1F
            rgb = b << 11 | g << 5 | r
            rgb = ((rgb & 0xFF) << 8) | ((rgb & 0xFF00) >> 8)
            output_file.write(struct.pack("H", rgb))


def main():
    parser = argparse.ArgumentParser(
        description="Convert a file from one format to another."
    )
    parser.add_argument(
        "-i",
        "--input",
        required=True,
        dest="input_file",
        help="Input file to be converted."
    )
    parser.add_argument(
        "-o",
        "--output",
        dest="output_file",
        help="Output file to be converted."
    )
    parser.add_argument(
        "-r",
        "--resize",
        nargs=2,
        type=int,
        dest="resize",
        help="resize image"
    )
    args = parser.parse_args()
    bgr565(args)


if __name__ == '__main__':
    main()

#!/usr/bin/env bash
# Downloads RoFormer models into a directory, for install.sh.
# Usage: download_roformer.sh DIRECTORY [bs4 sw6 mel4 vocals ...]  (without names: all)
# Check the terms of each model before using it, see README.md.
set -euo pipefail

target="${1:?usage: download_roformer.sh DIRECTORY [MODEL ...]}"
shift
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
zfturbo=https://github.com/ZFTurbo/Music-Source-Separation-Training

fetch() { # file url
  local file="$target/$1" sum
  sum=$(awk -v name="$1" '$2 == name { print $1 }' "$here/roformer.sha256")
  if [ ! -s "$file" ]; then
    mkdir -p "$(dirname "$file")"
    curl -fL --progress-bar -C - -o "$file.part" "$2"
    mv "$file.part" "$file"
  fi
  echo "$sum  $file" | shasum -a 256 -c --status || { echo "wrong checksum: $file" >&2; exit 1; }
}

for model in ${@:-bs4 sw6 mel4 vocals}; do
  case "$model" in
    bs4)
      base=$zfturbo/releases/download/v1.0.12
      fetch bs4/model.ckpt $base/model_bs_roformer_ep_17_sdr_9.6568.ckpt
      fetch bs4/config.yaml $base/config_bs_roformer_384_8_2_485100.yaml ;;
    sw6)
      base=https://huggingface.co/koidn/BS-RoFormer-SW-6stem/resolve/main
      fetch sw6/model.ckpt $base/BS-Rofo-SW-Fixed.ckpt
      fetch sw6/config.yaml $base/BS-Rofo-SW-Fixed.yaml ;;
    mel4)
      base=https://huggingface.co/SYH99999/MelBandRoformer4StemFTLarge/resolve/main
      fetch mel4/model.ckpt $base/MelBandRoformer4StemFTLarge.ckpt
      fetch mel4/config.yaml $base/config.yaml ;;
    vocals)
      fetch vocals/model.ckpt https://huggingface.co/KimberleyJSN/melbandroformer/resolve/main/MelBandRoformer.ckpt
      fetch vocals/config.yaml https://raw.githubusercontent.com/ZFTurbo/Music-Source-Separation-Training/main/configs/KimberleyJensen/config_vocals_mel_band_roformer_kj.yaml ;;
    *) echo "unknown model: $model" >&2; exit 1 ;;
  esac
done

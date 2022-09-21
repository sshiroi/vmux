# VMUX
This program was created in an effort to painlessly "remux" (anime) blurays into seperate files, but without acually creating a copy or destroying the source filestructure.
It originally achieved this by using a fuse filesystem and emulating mkv files containing raw video and audio frames. Essentially moving video decoding from the video player into this application. I eventually found out about EDL, in this mode the m2ts basically get stitched together by mpv. 

Currently only decrypted blurays are supported.

This is still only a proof of concept (The gui is not very user friendly, config format may change at any time and code quality is low) but it's at a point where its useful and comfortable for watching.

Fuse on windows under wsl2 should work in theory but no thorough testing has been done. 



# The modes you can use this program in
## MPV EDL (recommended)
- You can setup mpv as default program for edl and can watch like any standard encode
- The necessary chapters get extracted and an edl file is created
- The bdrom path will be hardcoded, there is still code thath enable symlinks (but its unused)
- (linux: You should also add a a new mime type for edl and associate it with mpv so you can just click on the files)
### pro:
- should work with most stuff and does not require indexing
- can display subtitle from bluray
### cons:
- since the mpv still hase some m2ts seeking problems (https://github.com/mpv-player/mpv/issues/9639 related),
and edl seems to always do a seek when opening the file, there are some m2ts files that will refuse to start at 0 when we give it the correct start, if we shift the stream by some (small ~30ms) constant time forewards everything seems to work, this is only really relevant for timing on subtitles you drag into mpv. 


## FUSE
- Uses ffms2 and emulates matroska files using fuse so they appear to be regular mkv files
### pro: 
- files are properly indexed like mkv remuxed m2ts
- you are in theory not bound to mpv
- files can be consumed by ffmpeg based programms example. alass-cli
### cons:
- some stuff is still buggy see known bugs
- since it's using ffms2 all files need to be indexed prior to viewing
- ffms2 seems to be very picky with its index files erroring out if any ffmpeg component has a version mismatch with the index
- this program needs to get all the metadata right (interlacing flags, color primaries and all this stuff)
- can't display subtitle from bluray (ffms2 does not support)
- linux only


## FTP
- Basically same as Fuse but without the filesystem part
- emulates an ftp server and launches mpv directly with the ftp protocol
- also works on windows 


# How to use the gui
- There is the gui and the vmux_main program. The main program reads the config and emulates the mkvs files using fuse. You can edit the config and export edl using the gui.
- You can add bdrom's under the Bdmvs tab
- You should hit inspect on the bd your currently working on
- (If you want to use ftp or fuse All their STREAMs should be indexed by going to BdStreams and hitting IndexAll)
- You add names to every title/playlist of the bd under the "Inspect" tab
- If you dont know/cant infer what is in what title you can hit OpenInVLC select something in the menu and look for the latest `HDMV_EVENT_PLAY_PL(4): x` in the VLC tab. x is the playlist number of the thing viewed.
- If you have alot of episodes stuffed into one clip you can observer `HDMV_EVENT_PLAY_PM(6): x` when selecting the individual episodes in the menu for cutting based on chapters (TODO: Are playmarks always equal to the chapers ?)
- You can use the big textbox for notes/episodes names etc
- You can also add chapter names to the respective titles if you want (currently they only appear in ftp or fuse mode)
- Once you gone through all the bds you want to create a folder. Add a folder under "Folders" Add folder
- Select the folder (and make sure "show" is checked only relevant for fuse or ftp)
- (You can now change/set the fileprefix all mkv's will recieve)
- You can now add FullPlaylist, ClipSplit and ClipIndex files to your folder
- You can do all of this from the Inspect panel
- Set the a index dir under Config or EBL Save Location depending on what you want to use and save the config
- Note: There is no auto save, save frequently there might still be crashes



# Extracts
## FullPlaylist
- You can export Fullplaylists as one file (useful for NCED, NCOP, ...)
- FUSE only: If there are multiple clips that don't have the same amount audio, everything thats more than the one with the fewest doesn't get included
- (you can rightclick the playlist header for quick add)

## ClipIndex
- Exports one clip from one playlist as one file
- useful for Menus or if you want give single episodes specific names

## ClipSplit
- Exports x clip's from one playlist as x files
- If you put {} in the name it gets replaced by a number starting at formatstart padded with 0 to formatminwidth)

## FromToChapter
- You can split playlists from and to chapter's (useful when multiple eps are shoved into one clip in one playlist)
- start an end are inclusive
- The first chapter starts at index 0 (might add an gui option that lets you choose between index 0 first or index 1 first)


## Building (general)
- download ipagp.ttf (example from https://github.com/hyoshiok/ttf-ipafont/blob/master/ipagp.ttf)
- make sure you have ffms 2.40 installed
- build the project like any other rust project with all the required dependencies installed ( ffms2, fuse, libbluray, (vlc, mpv) )
- You can use the provided `shell.nix` under nixos for all required dependecys (+rustup )

## Building (Windows msvc)
- download and install rustup (with msvc buildtools)
- download and install LLVM ( add LLVM to systempath )
- download ffms2
- download libbluray and dependencys 
- download mpv and vlc
- add mpv and vlc to PATH
- Add env variables FFMS_INCLUDE_DIR and FFMS_LIB_DIR
- Add env variable LIBBLURAY_INCLUDE_DIR pointing to libbluray include
- Add env variable LIBBLURAY_AND_LIBS_DIR pointing to a folder with all lib files from libbluray and dependencys
- `cargo build --bin vmux_gui --release`

## Links (Windows msvc):
- https://rustup.rs
- https://www.videolan.org/vlc/download-windows.html
- https://sourceforge.net/projects/mpv-player-windows/files/64bit/
- https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.6/LLVM-14.0.6-win64.exe
- https://github.com/FFMS/ffms2/releases/download/2.40/ffms2-2.40-msvc.7z
- https://github.com/ShiftMediaProject/libbluray/releases/download/1.3.1-2/libbluray_1.3.1-2_msvc17.zip
- https://github.com/ShiftMediaProject/bzip2/releases/download/bzip2-1.0.8-1/libbz2_bzip2-1.0.8-1_msvc17.zip
- https://github.com/ShiftMediaProject/fontconfig/releases/download/2.14.0-2/libfontconfig_2.14.0-2_msvc17.zip
- https://github.com/ShiftMediaProject/freetype2/releases/download/VER-2-12-1/libfreetype2_VER-2-12-1_msvc17.zip
- https://github.com/ShiftMediaProject/libiconv/releases/download/v1.17/libiconv_v1.17_msvc17.zip
- https://github.com/ShiftMediaProject/liblzma/releases/download/v5.2.5-1/liblzma_v5.2.5-1_msvc17.zip
- https://github.com/ShiftMediaProject/libxml2/releases/download/v2.9.14/libxml2_v2.9.14_msvc17.zip
- https://github.com/ShiftMediaProject/zlib/releases/download/v1.2.12/libzlib_v1.2.12_msvc17.zip


## Building ('Linux' / Ubuntu / wsl2)
- wsl2 only: Install Ubuntu 22.04 from windows store, older ubuntu version dont have new ffms2
- apt install curl
- install rust from rustup.rs
- `sudo apt install xorg gcc mpv vlc fuse libfuse-dev libclang-dev libbluray-dev libffms2-dev libbz2-dev`
- cargo build --release
- wsl2: If X doesn't work use VcXsrv and https://stackoverflow.com/questions/61110603/how-to-set-up-working-x11-forwarding-on-wsl2?answertab=trending#tab-top
- you need to run vmux_main under root in wsl to acces files from windows example: `sudo ./target/release/vmux_main --config /home/user/.vmux/config.json --mount /some_mountpoint --allow_other`
- when running under wsl you can acces the files from windows explorer under `\\wsl$\<distroname>/some_mountpoint`

## ffms2-rs build fails with dolby something
- Apparently they added fields in a struct, made no new release ubuntu has the new(git) version but nixos does not
- Go to vmux_lib/Cargo.toml and change to commit hash to include the dv fix


## Known bugs (fuse and ftp only)
- everything that isn't regular progressive 420 constant framerate video, may (currently) not work properly example: reallife extras, or some interlaced stuff.
- random video corruption (horizontal lines are shifted (and blended together from multiples frames?)) for ~10secs or so (rare only happend like 1-2 times per 12 episodes) (can't reproduce reliabley sadly) [1]
- random frame "corruption" when you have 2 programms access the file at different places (no idea whats going on,https://github.com/FFMS/ffms2/issues/403 might be related) might be related to [1]
- some filenames cause problems with file enumeration (readdir). mostly japanese only filenames
- extracting audio is slow ~1.4x (needed for alass to be useful)
- on some movie crashes vmux_main at the end due 'timestamp difference error'
- wav and y4m crash at end of file ( unmaintained code should just throw that out)

## Known bugs EDL 
- none yet


## Features I still want to implement eventually
- DVD support
- ability to set metadata ( audio language)
- Some kind of vapoursynth support 
- mux in external .srt and .ass
- mux audio from different bluray/clip
- bluray: cutout singular clip from playlist (would be useful when audio+commentary tracks are present and theres a copyright warning at the end with only 1 audio)
- find a way to replace ffms2 (l-smash?) or atleast pin every version of ffms2,avutil,avformat,...
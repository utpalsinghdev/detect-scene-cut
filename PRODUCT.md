# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

The owner uploads videos, cuts them at scene changes, and downloads the clips for use elsewhere.

## Product Purpose

Split one video into separate scene clips without adding decoded video or audio quality loss. Download every clip in one ZIP named after the source video.

## Capabilities and Constraints

React/TypeScript frontend and Rust backend. Streamed uploads without an application size cap. Adjustable scene sensitivity, source preview, scene seeking, background progress, ZIP download, and explicit deletion. FFV1 MKV video and PCM audio; output can be much larger than input. Subtitles and dynamic HDR metadata are not preserved. Processing behavior must remain unchanged during frontend redesign. User subsequently requested a visible Clear video action and an empty workspace after refresh. These clear selection only and do not delete server files; trash remains a separate destructive action.

## Operating Context

Private Tailscale access on the home server. User requested port 1900 and a reboot-persistent service. Existing uploads must be preserved.

## Brand Commitments

Keep the scene / split product name. User supplied a Cohere-inspired editorial design spec for the redesign: white canvas, deep-green product regions, tight large display type, restrained UI, and rounded media. It is a reference system, not permission to use Cohere's identity or proprietary fonts.

## Evidence on Hand

The functioning application and uploaded footage. No customer logos, testimonials, commercial metrics, or stock imagery supplied. Do not invent them.

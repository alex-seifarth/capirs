<!--
*** README template from https://raw.githubusercontent.com/OSKWalker/Best-README-Template/
-->

<!-- PROJECT SHIELDS -->

[comment]: <> ([![Contributors][contributors-shield]][contributors-url])

[comment]: <> ([![Forks][forks-shield]][forks-url])

[comment]: <> ([![Stargazers][stars-shield]][stars-url])

[comment]: <> ([![Issues][issues-shield]][issues-url])

[comment]: <> ([![MIT License][license-shield]][license-url])


<br />
<p align="center">
  <!--<a href="https://github.com/alex-seifarth/capirs">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a>
-->

<h3 align="center">CAPI for Rust</h3>

  <p align="center">
    Using vsomeip with Rust
    <br />
    <a href="https://github.com/alex-seifarth/capirs"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <!--<a href="https://github.com/alex-seifarth/capirs">View Demo</a> -->
    ·
    <a href="https://github.com/alex-seifarth/capirs/issues">Report Bug</a>
    ·
    <a href="https://github.com/alex-seifarth/capirs/issues">Request Feature</a>
  </p>
</p>



<!-- TABLE OF CONTENTS -->
<details open="open">
  <summary><h2 style="display: inline-block">Table of Contents</h2></summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgements">Acknowledgements</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project
This is actually a non-productive proof-of-concept repository how a CommonAPI for Rust 
could look like. This concerns the interface towards users of the API and also 
the internal architecture with respect to how vsomeip is being used and wrapped.

### Built With

* [rust]() 
* [cmake]()
* [gcc 10]()
* [vsomeip 3]()

## Getting Started

### Download/Clone the source
Get the source code from github by cloning the repo using ssh
```shell
git clone git@github.com:alex-seifarth/capirs.git
```
or https
```shell
git clone https://github.com/alex-seifarth/capirs.git
```
or download a source tarball
```shell
https://github.com/alex-seifarth/capirs/archive/refs/heads/master.zip
```

### Prerequisites
* Rust in an actual version (1.55, 1.56) with Cargo is required. See [https://www.rust-lang.org/](https://www.rust-lang.org/),
* vsomeip v3 must be available on the system so that cmake can find it [https://github.com/GENIVI/vsomeip](https://github.com/GENIVI/vsomeip).

### Building
The C/C++ part is integrated into the Rust build process and should not require 
extra attention. 

In the best case a 
```shell
cd <capirs-root-dir>
cargo build
```
should do everything - it should build the 
* static C/C++ library interfacing `vsomeip v3` shared library,
* build the `capirs` library linked statically to the before built C/C++ library 
* building the example code to test.

### Installation
There is actually no installation due to the proof-of-concept nature of the project.

### Running examples
The examples under ```<capirs-root>/examples``` are used to test the library.
They are build always with the library and could be run under 
```<capirs-root>/target/debug/<example-code>```.
It might be necessary to set ```LD_LIBRARY_PATH``` before the executable can be 
run, the environment variable should contain the path to the ```libvsomeip.so.3``` 
shared library.

## Roadmap
 - [x] Offer service provider
 - [x] Offer events
 - [x] Send Requests / Receive Requests
 - [x] Send Response / Receive Response (incl. error) / Session timeout
 - [x] Send events
 - [ ] Request/Subscribe events
 - [ ] Service stub creation by runtime
 - [ ] Service stub generation from FIDL/FDEPL file via ```pyfranca```
 - [ ] Proxy creation by runtime
 - [ ] Proxy generation from FIDL/FDEPL file via ```pyfranca```
 

<!-- CONTRIBUTING
## Contributing

Contributions are what make the open source community such an amazing place to be learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

-->


<!-- LICENSE -->
## License

Distributed under the Mozilla Public License v2. 
See `LICENSE` for more information.

<!-- CONTACT -->
## Contact

Alexander Seifarth - <email>

Project Link: [https://github.com/alex-seifarth/capirs](https://github.com/alex-seifarth/capirs)



<!-- ACKNOWLEDGEMENTS 
## Acknowledgements

* []()
* []()
* []()
-->





<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/alex-seifarth/repo.svg?style=for-the-badge
[contributors-url]: https://github.com/alex-seifarth/capirs/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/alex-seifarth/repo.svg?style=for-the-badge
[forks-url]: https://github.com/alex-seifarth/capirs/network/members
[stars-shield]: https://img.shields.io/github/stars/alex-seifarth/repo.svg?style=for-the-badge
[stars-url]: https://github.com/alex-seifarth/capirs/stargazers
[issues-shield]: https://img.shields.io/github/issues/alex-seifarth/repo.svg?style=for-the-badge
[issues-url]: https://github.com/alex-seifarth/capirs/issues
[license-shield]: https://img.shields.io/github/license/alex-seifarth/repo.svg?style=for-the-badge
[license-url]: https://github.com/alex-seifarth/capirs/blob/master/LICENSE

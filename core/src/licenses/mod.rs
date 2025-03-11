/*
    UrbXtract
    Copyright (C) 2025  Atheesh Thirumalairajan

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>. 
*/

pub fn get_license_string_short() -> String {
    return String::from("") +
        "UrbXtract  Copyright (C) 2025  Atheesh Thirumalairajan\n" +
        "This program comes with ABSOLUTELY NO WARRANTY; run with --license-info for details.\n" +
        "This is free software, and you are welcome to redistribute it\n" +
        "under certain conditions; run with --license-info for details.\n";
}

pub fn get_license_string_full() -> String {
    return String::from("") + 
    "UrbXtract\n" +
    "Copyright (C) 2025  Atheesh Thirumalairajan\n\n" +

    "This program is free software: you can redistribute it and/or modify\n" +
    "it under the terms of the GNU General Public License as published by\n" +
    "the Free Software Foundation, either version 3 of the License, or\n" +
    "(at your option) any later version.\n\n" +

    "This program is distributed in the hope that it will be useful,\n" +
    "but WITHOUT ANY WARRANTY; without even the implied warranty of\n" +
    "MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the\n" +
    "GNU General Public License for more details.\n\n" +

    "You should have received a copy of the GNU General Public License\n" +
    "along with this program.  If not, see <https://www.gnu.org/licenses/>.\n";
}
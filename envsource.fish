#! /usr/bin/fish
for line in (cat $argv | grep -v '^#')
    set item (string split -m 1 '=' $line | string trim)
    set -gx $item[1] $item[2]
    echo "Exported key $item[1]"
end
